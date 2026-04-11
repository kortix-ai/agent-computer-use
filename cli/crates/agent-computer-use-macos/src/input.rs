use agent_computer_use_core::action::{MouseButton, ScrollDirection};
use agent_computer_use_core::node::Point;
use agent_computer_use_core::{Error, Result};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, EventField};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use foreign_types::ForeignType;

const CGEVENT_TARGET_UNIX_PROCESS_ID: u32 = 89;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateScrollWheelEvent(
        source: *mut std::ffi::c_void,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
    ) -> *mut std::ffi::c_void;

    fn CGEventSetIntegerValueField(event: *mut std::ffi::c_void, field: u32, value: i64);
}

fn post_event_global(event: &CGEvent) {
    event.post(CGEventTapLocation::HID);
}

fn post_event_to_pid(event: &CGEvent, pid: i32) {
    unsafe {
        CGEventSetIntegerValueField(
            event.as_ptr() as *mut _,
            CGEVENT_TARGET_UNIX_PROCESS_ID,
            pid as i64,
        );
    }
    event.post(CGEventTapLocation::HID);
}

fn post_event(event: &CGEvent, target_pid: Option<i32>) {
    match target_pid {
        Some(pid) => post_event_to_pid(event, pid),
        None => post_event_global(event),
    }
}

pub fn stealth_activate<F, R>(app_name: &str, f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    let save_script = r#"tell application "System Events" to get name of first application process whose frontmost is true"#;
    let original_app = std::process::Command::new("osascript")
        .args(["-e", save_script])
        .output()
        .ok()
        .and_then(|o| {
            let name = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        });

    let activate_script = format!(r#"tell application "{app_name}" to activate"#);
    let _ = std::process::Command::new("osascript")
        .args(["-e", &activate_script])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(100));

    let result = f();

    if let Some(ref orig) = original_app {
        let restore_script = format!(r#"tell application "{orig}" to activate"#);
        let _ = std::process::Command::new("osascript")
            .args(["-e", &restore_script])
            .output();
    }

    result
}

pub fn click(point: Point, button: MouseButton, count: u32) -> Result<()> {
    click_impl(point, button, count, None)
}

pub fn click_to_pid(point: Point, button: MouseButton, count: u32, pid: i32) -> Result<()> {
    click_impl(point, button, count, Some(pid))
}

fn click_impl(
    point: Point,
    button: MouseButton,
    count: u32,
    target_pid: Option<i32>,
) -> Result<()> {
    let cg_point = CGPoint::new(point.x, point.y);
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    let move_event = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::MouseMoved,
        cg_point,
        CGMouseButton::Left,
    )
    .map_err(|_| Error::PlatformError {
        message: "failed to create mouse move event".into(),
    })?;
    post_event(&move_event, target_pid);

    let (down_type, up_type, cg_button) = match button {
        MouseButton::Left => (
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseUp,
            CGMouseButton::Left,
        ),
        MouseButton::Right => (
            CGEventType::RightMouseDown,
            CGEventType::RightMouseUp,
            CGMouseButton::Right,
        ),
        MouseButton::Middle => (
            CGEventType::OtherMouseDown,
            CGEventType::OtherMouseUp,
            CGMouseButton::Center,
        ),
    };

    const MULTI_CLICK_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

    for i in 0..count {
        if i > 0 {
            std::thread::sleep(MULTI_CLICK_DELAY);
        }

        let down = CGEvent::new_mouse_event(source.clone(), down_type, cg_point, cg_button)
            .map_err(|_| Error::PlatformError {
                message: "failed to create mouse down event".into(),
            })?;

        let up = CGEvent::new_mouse_event(source.clone(), up_type, cg_point, cg_button).map_err(
            |_| Error::PlatformError {
                message: "failed to create mouse up event".into(),
            },
        )?;

        down.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, (i + 1) as i64);
        up.set_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE, (i + 1) as i64);

        post_event(&down, target_pid);
        post_event(&up, target_pid);
    }

    Ok(())
}

pub fn move_mouse(point: Point) -> Result<()> {
    move_mouse_impl(point, None)
}

pub fn move_mouse_to_pid(point: Point, pid: i32) -> Result<()> {
    move_mouse_impl(point, Some(pid))
}

fn move_mouse_impl(point: Point, target_pid: Option<i32>) -> Result<()> {
    let cg_point = CGPoint::new(point.x, point.y);
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    let event = CGEvent::new_mouse_event(
        source,
        CGEventType::MouseMoved,
        cg_point,
        CGMouseButton::Left,
    )
    .map_err(|_| Error::PlatformError {
        message: "failed to create mouse move event".into(),
    })?;

    post_event(&event, target_pid);
    Ok(())
}

pub fn drag(from: Point, to: Point, target_pid: Option<i32>) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    let from_cg = CGPoint::new(from.x, from.y);
    let to_cg = CGPoint::new(to.x, to.y);

    let move_to_start = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::MouseMoved,
        from_cg,
        CGMouseButton::Left,
    )
    .map_err(|_| Error::PlatformError {
        message: "failed to create move event".into(),
    })?;
    post_event(&move_to_start, target_pid);
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mouse_down = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::LeftMouseDown,
        from_cg,
        CGMouseButton::Left,
    )
    .map_err(|_| Error::PlatformError {
        message: "failed to create mouse down event".into(),
    })?;
    post_event(&mouse_down, target_pid);
    std::thread::sleep(std::time::Duration::from_millis(100));

    let steps = 20;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let t_smooth = t * t * (3.0 - 2.0 * t);
        let x = from.x + (to.x - from.x) * t_smooth;
        let y = from.y + (to.y - from.y) * t_smooth;
        let pt = CGPoint::new(x, y);

        let drag_event = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDragged,
            pt,
            CGMouseButton::Left,
        )
        .map_err(|_| Error::PlatformError {
            message: "failed to create drag event".into(),
        })?;
        post_event(&drag_event, target_pid);
        std::thread::sleep(std::time::Duration::from_millis(8));
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mouse_up =
        CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, to_cg, CGMouseButton::Left)
            .map_err(|_| Error::PlatformError {
                message: "failed to create mouse up event".into(),
            })?;
    post_event(&mouse_up, target_pid);

    Ok(())
}

pub fn type_text(text: &str) -> Result<()> {
    type_text_impl(text, None)
}

fn type_text_impl(text: &str, target_pid: Option<i32>) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    for ch in text.chars() {
        let key_down = CGEvent::new_keyboard_event(source.clone(), 0, true).map_err(|_| {
            Error::PlatformError {
                message: format!("failed to create key down event for '{ch}'"),
            }
        })?;

        let key_up = CGEvent::new_keyboard_event(source.clone(), 0, false).map_err(|_| {
            Error::PlatformError {
                message: format!("failed to create key up event for '{ch}'"),
            }
        })?;

        let chars = [ch as u16];
        key_down.set_string_from_utf16_unchecked(&chars);
        key_up.set_string_from_utf16_unchecked(&chars);

        post_event(&key_down, target_pid);
        post_event(&key_up, target_pid);

        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    Ok(())
}

pub fn key_press(key_expr: &str) -> Result<()> {
    key_press_impl(key_expr, None)
}

pub fn key_press_to_pid(key_expr: &str, pid: i32) -> Result<()> {
    key_press_impl(key_expr, Some(pid))
}

fn key_press_impl(key_expr: &str, target_pid: Option<i32>) -> Result<()> {
    let parts: Vec<&str> = key_expr.split('+').collect();
    let (key, modifiers) = parts.split_last().ok_or_else(|| Error::PlatformError {
        message: format!("invalid key expression: '{key_expr}'"),
    })?;

    let mut flags = core_graphics::event::CGEventFlags::empty();
    for modifier in modifiers.iter() {
        match modifier.to_lowercase().as_str() {
            "cmd" | "command" | "super" => {
                flags |= core_graphics::event::CGEventFlags::CGEventFlagCommand;
            }
            "ctrl" | "control" => {
                flags |= core_graphics::event::CGEventFlags::CGEventFlagControl;
            }
            "alt" | "option" | "opt" => {
                flags |= core_graphics::event::CGEventFlags::CGEventFlagAlternate;
            }
            "shift" => {
                flags |= core_graphics::event::CGEventFlags::CGEventFlagShift;
            }
            unknown => {
                return Err(Error::PlatformError {
                    message: format!("unknown modifier: '{unknown}'"),
                });
            }
        }
    }

    let keycode = key_name_to_keycode(key)?;

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    let key_down = CGEvent::new_keyboard_event(source.clone(), keycode, true).map_err(|_| {
        Error::PlatformError {
            message: "failed to create key down event".into(),
        }
    })?;

    let key_up =
        CGEvent::new_keyboard_event(source, keycode, false).map_err(|_| Error::PlatformError {
            message: "failed to create key up event".into(),
        })?;

    key_down.set_flags(flags);
    key_up.set_flags(flags);

    post_event(&key_down, target_pid);
    post_event(&key_up, target_pid);

    Ok(())
}

pub fn scroll(direction: ScrollDirection, amount: u32) -> Result<()> {
    scroll_with_pid(direction, amount, None)
}

pub fn scroll_with_pid(
    direction: ScrollDirection,
    amount: u32,
    target_pid: Option<i32>,
) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
        Error::PlatformError {
            message: "failed to create CGEventSource".into(),
        }
    })?;

    let total_pixels = amount as i32 * 40;
    let steps = 8;
    let per_step = total_pixels / steps;

    let (base_dy, base_dx) = match direction {
        ScrollDirection::Up => (per_step, 0),
        ScrollDirection::Down => (-per_step, 0),
        ScrollDirection::Left => (0, per_step),
        ScrollDirection::Right => (0, -per_step),
    };

    for i in 0..steps {
        let event_ref = unsafe {
            CGEventCreateScrollWheelEvent(source.as_ptr() as *mut _, 1, 2, base_dy, base_dx)
        };

        if event_ref.is_null() {
            return Err(Error::PlatformError {
                message: "failed to create scroll event".into(),
            });
        }

        let event = unsafe { CGEvent::from_ptr(event_ref as *mut _) };

        if i == 0 {
            unsafe {
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 99, 1); // kCGScrollWheelEventIsContinuous
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 123, 2); // NSEventPhaseBegan
            }
        } else if i == steps - 1 {
            unsafe {
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 99, 1);
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 123, 4); // NSEventPhaseEnded
            }
        } else {
            unsafe {
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 99, 1);
                CGEventSetIntegerValueField(event.as_ptr() as *mut _, 123, 2); // NSEventPhaseChanged
            }
        }

        post_event(&event, target_pid);
        std::mem::forget(event);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}

fn key_name_to_keycode(name: &str) -> Result<u16> {
    let code = match name.to_lowercase().as_str() {
        "a" => 0x00,
        "b" => 0x0B,
        "c" => 0x08,
        "d" => 0x02,
        "e" => 0x0E,
        "f" => 0x03,
        "g" => 0x05,
        "h" => 0x04,
        "i" => 0x22,
        "j" => 0x26,
        "k" => 0x28,
        "l" => 0x25,
        "m" => 0x2E,
        "n" => 0x2D,
        "o" => 0x1F,
        "p" => 0x23,
        "q" => 0x0C,
        "r" => 0x0F,
        "s" => 0x01,
        "t" => 0x11,
        "u" => 0x20,
        "v" => 0x09,
        "w" => 0x0D,
        "x" => 0x07,
        "y" => 0x10,
        "z" => 0x06,
        "0" => 0x1D,
        "1" => 0x12,
        "2" => 0x13,
        "3" => 0x14,
        "4" => 0x15,
        "5" => 0x17,
        "6" => 0x16,
        "7" => 0x1A,
        "8" => 0x1C,
        "9" => 0x19,
        "return" | "enter" => 0x24,
        "escape" | "esc" => 0x35,
        "tab" => 0x30,
        "space" => 0x31,
        "backspace" | "delete" => 0x33,
        "forwarddelete" => 0x75,
        "up" => 0x7E,
        "down" => 0x7D,
        "left" => 0x7B,
        "right" => 0x7C,
        "home" => 0x73,
        "end" => 0x77,
        "pageup" => 0x74,
        "pagedown" => 0x79,
        "f1" => 0x7A,
        "f2" => 0x78,
        "f3" => 0x63,
        "f4" => 0x76,
        "f5" => 0x60,
        "f6" => 0x61,
        "f7" => 0x62,
        "f8" => 0x64,
        "f9" => 0x65,
        "f10" => 0x6D,
        "f11" => 0x67,
        "f12" => 0x6F,
        "-" | "minus" => 0x1B,
        "=" | "equal" => 0x18,
        "[" | "leftbracket" => 0x21,
        "]" | "rightbracket" => 0x1E,
        "\\" | "backslash" => 0x2A,
        ";" | "semicolon" => 0x29,
        "'" | "quote" => 0x27,
        "," | "comma" => 0x2B,
        "." | "period" => 0x2F,
        "/" | "slash" => 0x2C,
        "`" | "grave" => 0x32,
        other => {
            return Err(Error::PlatformError {
                message: format!("unknown key: '{other}'"),
            });
        }
    };
    Ok(code)
}
