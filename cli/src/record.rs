use agent_computer_use_core::action::Action;
use agent_computer_use_core::node::{AccessibilityNode, Role};
use agent_computer_use_core::{Error, Platform, Result};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const IDLE_FLUSH_MS: u64 = 600;

#[derive(Serialize)]
struct Meta {
    version: u32,
    schema: &'static str,
    started_at_unix_ms: u128,
    platform: String,
    hostname: Option<String>,
    capture_moves: bool,
    frames_enabled: bool,
    frame_every_ms: u64,
    ocr_enabled: bool,
}

#[derive(Serialize, Default)]
struct Target {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bbox: Option<[f64; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secure: Option<bool>,
}

#[derive(Serialize)]
struct Event {
    t_ms: u64,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    button: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frame: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<Target>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ocr_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ocr_scene: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clipboard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scene: Option<Vec<SceneItem>>,
}

#[derive(Serialize)]
struct SceneItem {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

pub struct RecordOptions {
    pub dir: PathBuf,
    pub frames_enabled: bool,
    pub frame_every_ms: u64,
    pub capture_moves: bool,
}

enum RawEvent {
    MouseDown {
        t_ms: u64,
        button: &'static str,
    },
    MouseUp {
        t_ms: u64,
        button: &'static str,
    },
    MouseMove {
        t_ms: u64,
        x: f64,
        y: f64,
    },
    KeyDown {
        t_ms: u64,
        key: String,
    },
    KeyUp {
        #[allow(dead_code)]
        t_ms: u64,
        key: String,
    },
    Wheel {
        t_ms: u64,
        dx: f64,
        dy: f64,
    },
}

pub async fn record(_wrapped: &dyn Platform, opts: RecordOptions) -> Result<()> {
    // Use the raw native platform. The passed-in platform may be wrapped in
    // ElectronAwarePlatform which auto-relaunches Chrome/Electron apps with CDP
    // when queried — a destructive side effect the recorder must avoid.
    let native = native_platform();
    let platform: &dyn Platform = &native;
    fs::create_dir_all(&opts.dir)?;
    let frames_dir = opts.dir.join("frames");
    if opts.frames_enabled {
        fs::create_dir_all(&frames_dir)?;
    }

    let ocr_enabled = tesseract_available();

    let meta = Meta {
        version: 1,
        schema: "agent-cu.record.v1",
        started_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        platform: platform.platform_name().to_string(),
        hostname: hostname_best_effort(),
        capture_moves: opts.capture_moves,
        frames_enabled: opts.frames_enabled,
        frame_every_ms: opts.frame_every_ms,
        ocr_enabled,
    };
    fs::write(
        opts.dir.join("meta.json"),
        serde_json::to_vec_pretty(&meta)?,
    )?;

    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).map_err(|e| {
            Error::PlatformError {
                message: format!("failed to install ctrl-c handler: {e}"),
            }
        })?;
    }

    let (tx, rx) = mpsc::channel::<RawEvent>();
    let start = Instant::now();

    std::thread::spawn(move || {
        let _ = rdev::listen(move |event| {
            let t_ms = start.elapsed().as_millis() as u64;
            if let Some(ev) = translate(event, t_ms) {
                let _ = tx.send(ev);
            }
        });
    });

    let events_path = opts.dir.join("events.jsonl");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&events_path)?;
    let writer = BufWriter::new(file);

    let mut session = Session {
        writer,
        opts,
        frames_dir,
        ocr_enabled,
        last_mouse: None,
        frame_counter: 0,
        frames_written: 0,
        last_frame_ms: 0,
        modifiers: HashSet::new(),
        typing_buffer: String::new(),
        typing_started_ms: 0,
        typing_target: None,
        typing_secure: false,
        last_input_ms: 0,
        last_ocr_scene_ms: 0,
        last_wheel_flush_ms: 0,
        wheel_accum_dx: 0.0,
        wheel_accum_dy: 0.0,
        wheel_accum_x: 0.0,
        wheel_accum_y: 0.0,
        wheel_count: 0,
    };

    eprintln!(
        "recording → {} (ocr: {}, Ctrl+C to stop)",
        session.opts.dir.display(),
        if ocr_enabled { "on" } else { "off" }
    );

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(raw) => {
                session.handle(platform, raw).await;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let t_ms = start.elapsed().as_millis() as u64;
                session.maybe_idle_flush(t_ms).await;
                session.maybe_wheel_flush(platform, t_ms).await;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Final flushes.
    let t_ms = start.elapsed().as_millis() as u64;
    session.flush_typing(t_ms, Some(platform)).await;
    session.flush_wheel(platform, t_ms).await;
    session.writer.flush().ok();
    eprintln!(
        "\nsaved {} frames, {} (events.jsonl)",
        session.frames_written,
        events_path.display()
    );
    Ok(())
}

struct Session {
    writer: BufWriter<File>,
    opts: RecordOptions,
    frames_dir: PathBuf,
    ocr_enabled: bool,

    last_mouse: Option<(f64, f64)>,
    frame_counter: u64,
    frames_written: u64,
    last_frame_ms: u64,

    modifiers: HashSet<String>,

    // Typing buffer: chars collected between flushes.
    typing_buffer: String,
    typing_started_ms: u64,
    typing_target: Option<Target>,
    typing_secure: bool,
    last_input_ms: u64,

    // OCR scene throttle.
    last_ocr_scene_ms: u64,

    // Wheel coalescing.
    last_wheel_flush_ms: u64,
    wheel_accum_dx: f64,
    wheel_accum_dy: f64,
    wheel_accum_x: f64,
    wheel_accum_y: f64,
    wheel_count: u32,
}

impl Session {
    async fn handle(&mut self, platform: &dyn Platform, raw: RawEvent) {
        match raw {
            RawEvent::MouseMove { t_ms, x, y } => {
                self.last_mouse = Some((x, y));
                if self.opts.capture_moves {
                    self.write_event(Event {
                        t_ms,
                        kind: "mouse_move",
                        button: None,
                        key: None,
                        text: None,
                        x: Some(x),
                        y: Some(y),
                        delta_x: None,
                        delta_y: None,
                        frame: None,
                        app: None,
                        window_title: None,
                        target: None,
                        ocr_text: None,
                        clipboard: None,
                        ocr_scene: None,
                        scene: None,
                    });
                }
            }
            RawEvent::MouseDown { t_ms, button } => {
                // Mouse click breaks a typing session: flush the buffer first.
                self.flush_wheel(platform, t_ms).await;
                self.flush_typing(t_ms, Some(platform)).await;

                let (x, y) = self.last_mouse.unwrap_or((0.0, 0.0));
                let target = resolve_target(platform, x, y).await;
                let ocr_text = if target.is_none() {
                    self.ocr_region(platform, t_ms, x, y).await
                } else {
                    None
                };
                let frame = self.maybe_capture(platform, t_ms, true).await;
                let app = frontmost_app_name(platform).await;
                let window_title = frontmost_window_title(platform, app.as_deref()).await;
                let scene = scene_snapshot(platform, app.as_deref()).await;
                let ocr_scene = self.maybe_ocr_scene(t_ms, frame.as_deref()).await;

                self.write_event(Event {
                    t_ms,
                    kind: "mouse_down",
                    button: Some(button),
                    key: None,
                    text: None,
                    x: Some(x),
                    y: Some(y),
                    delta_x: None,
                    delta_y: None,
                    frame,
                    app,
                    window_title,
                    target,
                    ocr_text,
                    ocr_scene,
                    clipboard: None,
                    scene,
                });
                self.last_input_ms = t_ms;
            }
            RawEvent::MouseUp { t_ms, button } => {
                let (x, y) = self.last_mouse.unwrap_or((0.0, 0.0));
                self.write_event(Event {
                    t_ms,
                    kind: "mouse_up",
                    button: Some(button),
                    key: None,
                    text: None,
                    x: Some(x),
                    y: Some(y),
                    delta_x: None,
                    delta_y: None,
                    frame: None,
                    app: None,
                    window_title: None,
                    target: None,
                    ocr_text: None,
                    clipboard: None,
                    ocr_scene: None,
                    scene: None,
                });
            }
            RawEvent::KeyDown { t_ms, key } => {
                self.flush_wheel(platform, t_ms).await;

                // Track modifier state.
                if is_modifier(&key) {
                    self.modifiers.insert(key.clone());
                    return;
                }

                let cmd_held = self.cmd_held();
                // Shortcut capture: Cmd+C, Cmd+X, Cmd+V etc. get their own event with
                // clipboard contents attached (for copy/cut) or context (for paste).
                if cmd_held && is_clipboard_op(&key) {
                    self.flush_typing(t_ms, Some(platform)).await;
                    // Give the OS a beat to populate the pasteboard on copy/cut.
                    tokio::time::sleep(Duration::from_millis(40)).await;
                    let clipboard = read_clipboard();
                    let app = frontmost_app_name(platform).await;
                    self.write_event(Event {
                        t_ms,
                        kind: if key == "KeyV" { "paste" } else { "copy" },
                        button: None,
                        key: Some(key),
                        text: None,
                        x: None,
                        y: None,
                        delta_x: None,
                        delta_y: None,
                        frame: None,
                        app,
                        window_title: None,
                        target: None,
                        ocr_text: None,
                        ocr_scene: None,
                        clipboard,
                        scene: None,
                    });
                    return;
                }

                // Special keys (Enter, Tab, Escape, arrows) get emitted as raw key events
                // and flush any pending typed text.
                if is_special(&key) || cmd_held || self.ctrl_held() {
                    self.flush_typing(t_ms, Some(platform)).await;
                    let app = frontmost_app_name(platform).await;
                    let frame = self.maybe_capture(platform, t_ms, true).await;
                    self.write_event(Event {
                        t_ms,
                        kind: "key_down",
                        button: None,
                        key: Some(format_key_with_modifiers(&key, &self.modifiers)),
                        text: None,
                        x: None,
                        y: None,
                        delta_x: None,
                        delta_y: None,
                        frame,
                        app,
                        window_title: None,
                        target: None,
                        ocr_text: None,
                        clipboard: None,
                        ocr_scene: None,
                        scene: None,
                    });
                    self.last_input_ms = t_ms;
                    return;
                }

                // Plain text: append to buffer.
                if let Some(ch) = key_to_char(&key, self.shift_held()) {
                    if self.typing_buffer.is_empty() {
                        self.typing_started_ms = t_ms;
                        if let Ok(node) = platform.focused().await {
                            self.typing_secure = is_secure_field(&node);
                            self.typing_target = Some(node_to_target(&node));
                        }
                    }
                    self.typing_buffer.push(ch);
                    self.last_input_ms = t_ms;
                }
            }
            RawEvent::KeyUp { t_ms: _, key } => {
                if is_modifier(&key) {
                    self.modifiers.remove(&key);
                }
            }
            RawEvent::Wheel { t_ms, dx, dy } => {
                let (x, y) = self.last_mouse.unwrap_or((0.0, 0.0));
                if self.wheel_count == 0 {
                    self.last_wheel_flush_ms = t_ms;
                    self.wheel_accum_x = x;
                    self.wheel_accum_y = y;
                }
                self.wheel_accum_dx += dx;
                self.wheel_accum_dy += dy;
                self.wheel_count += 1;
                // If accumulated for > 250ms, flush as a single event.
                if t_ms.saturating_sub(self.last_wheel_flush_ms) >= 250 {
                    self.flush_wheel(platform, t_ms).await;
                }
            }
        }
    }

    async fn maybe_idle_flush(&mut self, t_ms: u64) {
        if !self.typing_buffer.is_empty()
            && t_ms.saturating_sub(self.last_input_ms) >= IDLE_FLUSH_MS
        {
            self.flush_typing(t_ms, None).await;
        }
    }

    async fn maybe_wheel_flush(&mut self, platform: &dyn Platform, t_ms: u64) {
        if self.wheel_count > 0 && t_ms.saturating_sub(self.last_wheel_flush_ms) >= 250 {
            self.flush_wheel(platform, t_ms).await;
        }
    }

    async fn flush_wheel(&mut self, platform: &dyn Platform, t_ms: u64) {
        if self.wheel_count == 0 {
            return;
        }
        let app = frontmost_app_name(platform).await;
        let frame = self.maybe_capture(platform, t_ms, true).await;
        self.write_event(Event {
            t_ms,
            kind: "wheel",
            button: None,
            key: None,
            text: None,
            x: Some(self.wheel_accum_x),
            y: Some(self.wheel_accum_y),
            delta_x: Some(self.wheel_accum_dx),
            delta_y: Some(self.wheel_accum_dy),
            frame,
            app,
            window_title: None,
            target: None,
            ocr_text: None,
            clipboard: None,
            ocr_scene: None,
            scene: None,
        });
        self.wheel_count = 0;
        self.wheel_accum_dx = 0.0;
        self.wheel_accum_dy = 0.0;
    }

    async fn flush_typing(&mut self, t_ms: u64, platform: Option<&dyn Platform>) {
        if self.typing_buffer.is_empty() {
            return;
        }
        let started = self.typing_started_ms;
        let buf = std::mem::take(&mut self.typing_buffer);
        let target = self.typing_target.take();
        let secure = self.typing_secure;
        self.typing_secure = false;

        let text = if secure {
            Some(format!("[REDACTED {} chars]", buf.chars().count()))
        } else {
            Some(buf)
        };
        let app = match platform {
            Some(p) => frontmost_app_name(p).await,
            None => None,
        };
        self.write_event(Event {
            t_ms: started,
            kind: "typed",
            button: None,
            key: None,
            text,
            x: None,
            y: None,
            delta_x: None,
            delta_y: None,
            frame: None,
            app,
            window_title: None,
            target,
            ocr_text: None,
            clipboard: None,
            ocr_scene: None,
            scene: None,
        });
        let _ = t_ms;
    }

    async fn maybe_capture(
        &mut self,
        platform: &dyn Platform,
        t_ms: u64,
        significant: bool,
    ) -> Option<String> {
        if !self.opts.frames_enabled || !significant {
            return None;
        }
        if t_ms.saturating_sub(self.last_frame_ms) < self.opts.frame_every_ms {
            return None;
        }
        self.last_frame_ms = t_ms;
        self.frame_counter += 1;
        let rel = format!("frames/{:06}.png", self.frame_counter);
        let abs = self
            .frames_dir
            .join(format!("{:06}.png", self.frame_counter));
        if platform
            .perform(&Action::Screenshot {
                path: Some(abs.to_string_lossy().to_string()),
                app: None,
            })
            .await
            .is_ok()
        {
            self.frames_written += 1;
            Some(rel)
        } else {
            None
        }
    }

    async fn ocr_region(
        &self,
        _platform: &dyn Platform,
        t_ms: u64,
        x: f64,
        y: f64,
    ) -> Option<String> {
        if !self.ocr_enabled || !self.opts.frames_enabled {
            return None;
        }
        // Crop from the most recent frame if available.
        let latest = self
            .frames_dir
            .join(format!("{:06}.png", self.frame_counter));
        if !latest.exists() {
            return None;
        }
        let _ = t_ms;
        ocr_crop_around(&latest, x, y)
    }

    /// Full-frame OCR. Expensive (~500ms–1s). Throttled to one per 2s.
    async fn maybe_ocr_scene(&mut self, t_ms: u64, frame: Option<&str>) -> Option<String> {
        if !self.ocr_enabled {
            return None;
        }
        let frame_rel = frame?;
        if t_ms.saturating_sub(self.last_ocr_scene_ms) < 2000 {
            return None;
        }
        let abs = self.opts.dir.join(frame_rel);
        if !abs.exists() {
            return None;
        }
        self.last_ocr_scene_ms = t_ms;
        ocr_full(&abs)
    }

    fn write_event(&mut self, ev: Event) {
        if let Ok(line) = serde_json::to_string(&ev) {
            let _ = writeln!(self.writer, "{line}");
            let _ = self.writer.flush();
        }
    }

    fn cmd_held(&self) -> bool {
        self.modifiers.contains("MetaLeft") || self.modifiers.contains("MetaRight")
    }
    fn ctrl_held(&self) -> bool {
        self.modifiers.contains("ControlLeft") || self.modifiers.contains("ControlRight")
    }
    fn shift_held(&self) -> bool {
        self.modifiers.contains("ShiftLeft") || self.modifiers.contains("ShiftRight")
    }
}

fn translate(event: rdev::Event, t_ms: u64) -> Option<RawEvent> {
    use rdev::EventType::*;
    match event.event_type {
        ButtonPress(b) => Some(RawEvent::MouseDown {
            t_ms,
            button: button_name(b),
        }),
        ButtonRelease(b) => Some(RawEvent::MouseUp {
            t_ms,
            button: button_name(b),
        }),
        MouseMove { x, y } => Some(RawEvent::MouseMove { t_ms, x, y }),
        KeyPress(k) => Some(RawEvent::KeyDown {
            t_ms,
            key: format!("{:?}", k),
        }),
        KeyRelease(k) => Some(RawEvent::KeyUp {
            t_ms,
            key: format!("{:?}", k),
        }),
        Wheel { delta_x, delta_y } => Some(RawEvent::Wheel {
            t_ms,
            dx: delta_x as f64,
            dy: delta_y as f64,
        }),
    }
}

fn button_name(b: rdev::Button) -> &'static str {
    match b {
        rdev::Button::Left => "left",
        rdev::Button::Right => "right",
        rdev::Button::Middle => "middle",
        rdev::Button::Unknown(_) => "other",
    }
}

fn is_modifier(k: &str) -> bool {
    matches!(
        k,
        "ShiftLeft"
            | "ShiftRight"
            | "ControlLeft"
            | "ControlRight"
            | "Alt"
            | "AltGr"
            | "MetaLeft"
            | "MetaRight"
            | "Function"
            | "CapsLock"
    )
}

fn is_special(k: &str) -> bool {
    matches!(
        k,
        "Return"
            | "Tab"
            | "Escape"
            | "Backspace"
            | "Delete"
            | "UpArrow"
            | "DownArrow"
            | "LeftArrow"
            | "RightArrow"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
            | "F1"
            | "F2"
            | "F3"
            | "F4"
            | "F5"
            | "F6"
            | "F7"
            | "F8"
            | "F9"
            | "F10"
            | "F11"
            | "F12"
    )
}

fn is_clipboard_op(k: &str) -> bool {
    matches!(k, "KeyC" | "KeyX" | "KeyV")
}

fn format_key_with_modifiers(k: &str, mods: &HashSet<String>) -> String {
    let mut parts = Vec::new();
    if mods.contains("MetaLeft") || mods.contains("MetaRight") {
        parts.push("cmd");
    }
    if mods.contains("ControlLeft") || mods.contains("ControlRight") {
        parts.push("ctrl");
    }
    if mods.contains("Alt") || mods.contains("AltGr") {
        parts.push("alt");
    }
    if mods.contains("ShiftLeft") || mods.contains("ShiftRight") {
        parts.push("shift");
    }
    parts.push(k);
    parts.join("+")
}

/// Map rdev key debug names to their characters. Incomplete but covers common text.
fn key_to_char(key: &str, shift: bool) -> Option<char> {
    let c = match key {
        "Space" => return Some(' '),
        "KeyA" => 'a',
        "KeyB" => 'b',
        "KeyC" => 'c',
        "KeyD" => 'd',
        "KeyE" => 'e',
        "KeyF" => 'f',
        "KeyG" => 'g',
        "KeyH" => 'h',
        "KeyI" => 'i',
        "KeyJ" => 'j',
        "KeyK" => 'k',
        "KeyL" => 'l',
        "KeyM" => 'm',
        "KeyN" => 'n',
        "KeyO" => 'o',
        "KeyP" => 'p',
        "KeyQ" => 'q',
        "KeyR" => 'r',
        "KeyS" => 's',
        "KeyT" => 't',
        "KeyU" => 'u',
        "KeyV" => 'v',
        "KeyW" => 'w',
        "KeyX" => 'x',
        "KeyY" => 'y',
        "KeyZ" => 'z',
        "Num0" => return Some(if shift { ')' } else { '0' }),
        "Num1" => return Some(if shift { '!' } else { '1' }),
        "Num2" => return Some(if shift { '@' } else { '2' }),
        "Num3" => return Some(if shift { '#' } else { '3' }),
        "Num4" => return Some(if shift { '$' } else { '4' }),
        "Num5" => return Some(if shift { '%' } else { '5' }),
        "Num6" => return Some(if shift { '^' } else { '6' }),
        "Num7" => return Some(if shift { '&' } else { '7' }),
        "Num8" => return Some(if shift { '*' } else { '8' }),
        "Num9" => return Some(if shift { '(' } else { '9' }),
        "Minus" => return Some(if shift { '_' } else { '-' }),
        "Equal" => return Some(if shift { '+' } else { '=' }),
        "LeftBracket" => return Some(if shift { '{' } else { '[' }),
        "RightBracket" => return Some(if shift { '}' } else { ']' }),
        "SemiColon" => return Some(if shift { ':' } else { ';' }),
        "Quote" => return Some(if shift { '"' } else { '\'' }),
        "BackSlash" => return Some(if shift { '|' } else { '\\' }),
        "Comma" => return Some(if shift { '<' } else { ',' }),
        "Dot" => return Some(if shift { '>' } else { '.' }),
        "Slash" => return Some(if shift { '?' } else { '/' }),
        "BackQuote" => return Some(if shift { '~' } else { '`' }),
        "Enter" => return Some('\n'),
        _ => return None,
    };
    Some(if shift { c.to_ascii_uppercase() } else { c })
}

async fn resolve_target(platform: &dyn Platform, x: f64, y: f64) -> Option<Target> {
    let node = platform.element_at_point(x, y).await.ok().flatten()?;
    Some(node_to_target(&node))
}

fn node_to_target(node: &AccessibilityNode) -> Target {
    let bbox = match (node.position, node.size) {
        (Some(p), Some(s)) => Some([p.x, p.y, s.width, s.height]),
        _ => None,
    };
    let secure = is_secure_field(node).then_some(true);
    Target {
        role: Some(role_str(&node.role).to_string()),
        name: node.name.clone(),
        value: if is_secure_field(node) {
            None
        } else {
            node.value.clone()
        },
        id: node.id.clone(),
        bbox,
        secure,
    }
}

fn is_secure_field(node: &AccessibilityNode) -> bool {
    matches!(node.role, Role::SecureTextField)
}

fn role_str(role: &Role) -> &'static str {
    // Rely on serde's camelCase rename; fall back to Debug if serialization fails.
    serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
        .unwrap_or("unknown")
}

async fn frontmost_app_name(platform: &dyn Platform) -> Option<String> {
    platform
        .applications()
        .await
        .ok()
        .and_then(|apps| apps.into_iter().find(|a| a.frontmost).map(|a| a.name))
}

async fn frontmost_window_title(platform: &dyn Platform, app: Option<&str>) -> Option<String> {
    let app = app?;
    platform.windows(Some(app)).await.ok().and_then(|ws| {
        ws.into_iter()
            .find(|w| w.frontmost.unwrap_or(false))
            .map(|w| w.title)
    })
}

/// Compact "scene" of the frontmost window: text-bearing interactive elements,
/// capped. Gives downstream consumers the surrounding context an element was
/// clicked in — universal, no app-specific knowledge.
async fn scene_snapshot(platform: &dyn Platform, app: Option<&str>) -> Option<Vec<SceneItem>> {
    let app = app?;
    let tree = platform.tree(Some(app), Some(8)).await.ok()?;
    let out = walk_scene_bfs(&tree, 150);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Breadth-first walk over the tree so top-level siblings (toolbar, sidebar,
/// main content) each get representation before any one subtree dominates.
fn walk_scene_bfs(root: &AccessibilityNode, cap: usize) -> Vec<SceneItem> {
    use std::collections::VecDeque;
    let mut out = Vec::with_capacity(cap);
    let mut queue: VecDeque<&AccessibilityNode> = VecDeque::new();
    queue.push_back(root);
    while let Some(node) = queue.pop_front() {
        if out.len() >= cap {
            break;
        }
        let has_text = node.name.as_deref().is_some_and(|s| !s.trim().is_empty())
            || node.value.as_deref().is_some_and(|s| !s.trim().is_empty());
        // Skip menu bar and menu items — macOS exposes the entire menu structure
        // via AX even when menus are closed, which would flood the scene with
        // noise. If the user actually clicks a menu, `target` captures it.
        if has_text
            && !matches!(
                node.role,
                Role::Application | Role::Window | Role::MenuBar | Role::MenuItem
            )
            && !is_menu_noise(&node.role)
        {
            let role = serde_json::to_value(&node.role)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".into());
            out.push(SceneItem {
                role,
                name: node.name.clone().filter(|s| !s.trim().is_empty()),
                value: node
                    .value
                    .as_ref()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| truncate(s, 200)),
            });
        }
        for child in &node.children {
            queue.push_back(child);
        }
    }
    out
}

fn is_menu_noise(role: &Role) -> bool {
    if let Role::Other(s) = role {
        let s = s.to_ascii_lowercase();
        s.contains("menuitem") || s.contains("menubaritem")
    } else {
        false
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max).collect();
        t.push('…');
        t
    }
}

fn read_clipboard() -> Option<String> {
    let mut cb = arboard::Clipboard::new().ok()?;
    cb.get_text().ok()
}

fn tesseract_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Crop a 400x120 region around (x, y) from `frame`, run tesseract, return text.
/// Uses `sips` for cropping on macOS (available out of the box).
fn ocr_full(frame: &Path) -> Option<String> {
    let out = Command::new("tesseract")
        .args([frame.to_string_lossy().as_ref(), "-", "-l", "eng"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(truncate(&text, 4000))
    }
}

fn ocr_crop_around(frame: &Path, x: f64, y: f64) -> Option<String> {
    let tmp = std::env::temp_dir().join(format!(
        "agent-cu-ocr-{}.png",
        std::process::id() as u64 ^ (x as u64).wrapping_mul(31) ^ (y as u64)
    ));
    let w = 400i32;
    let h = 120i32;
    let cx = (x as i32 - w / 2).max(0);
    let cy = (y as i32 - h / 2).max(0);

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("sips")
            .args([
                "-c",
                &h.to_string(),
                &w.to_string(),
                "--cropOffset",
                &cy.to_string(),
                &cx.to_string(),
                frame.to_string_lossy().as_ref(),
                "--out",
                tmp.to_string_lossy().as_ref(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok()?;
        if !status.success() {
            return None;
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Non-macOS: skip cropping, OCR the full frame (slower but works).
        std::fs::copy(frame, &tmp).ok()?;
        let _ = (cx, cy, w, h);
    }

    let out = Command::new("tesseract")
        .args([tmp.to_string_lossy().as_ref(), "-", "-l", "eng"])
        .output()
        .ok()?;
    let _ = std::fs::remove_file(&tmp);
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn native_platform() -> impl Platform {
    #[cfg(target_os = "macos")]
    {
        agent_computer_use_macos::MacOSPlatform::new()
    }
    #[cfg(target_os = "linux")]
    {
        agent_computer_use_linux::LinuxPlatform::new()
    }
    #[cfg(target_os = "windows")]
    {
        agent_computer_use_windows::WindowsPlatform::new()
    }
}

fn hostname_best_effort() -> Option<String> {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
}
