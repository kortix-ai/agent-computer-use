use agent_computer_use_core::action::{MouseButton, ScrollDirection};
use agent_computer_use_core::node::Point;
use agent_computer_use_core::{Error, Result};
use std::process::Command;

pub fn click(point: Point, button: MouseButton, count: u32) -> Result<()> {
    move_mouse(point)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Right => "3",
        MouseButton::Middle => "2",
    };

    for _ in 0..count {
        run_xdotool(&["click", btn])?;
    }

    Ok(())
}

pub fn move_mouse(point: Point) -> Result<()> {
    run_xdotool(&[
        "mousemove",
        "--sync",
        &(point.x as i32).to_string(),
        &(point.y as i32).to_string(),
    ])
}

pub fn type_text(text: &str) -> Result<()> {
    run_xdotool(&["type", "--clearmodifiers", "--delay", "15", text])
}

pub fn key_press(key_expr: &str) -> Result<()> {
    let translated = translate_key(key_expr);
    run_xdotool(&["key", "--clearmodifiers", &translated])
}

pub fn scroll(direction: ScrollDirection, amount: u32) -> Result<()> {
    let btn = match direction {
        ScrollDirection::Up => "4",
        ScrollDirection::Down => "5",
        ScrollDirection::Left => "6",
        ScrollDirection::Right => "7",
    };

    for _ in 0..amount {
        run_xdotool(&["click", btn])?;
    }

    Ok(())
}

pub fn mouse_down(point: Point, button: MouseButton) -> Result<()> {
    move_mouse(point)?;
    std::thread::sleep(std::time::Duration::from_millis(10));

    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Right => "3",
        MouseButton::Middle => "2",
    };

    run_xdotool(&["mousedown", btn])
}

pub fn mouse_up(button: MouseButton) -> Result<()> {
    let btn = match button {
        MouseButton::Left => "1",
        MouseButton::Right => "3",
        MouseButton::Middle => "2",
    };

    run_xdotool(&["mouseup", btn])
}

pub fn drag(from: Point, to: Point) -> Result<()> {
    mouse_down(from, MouseButton::Left)?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let steps = 20;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = from.x + (to.x - from.x) * t;
        let y = from.y + (to.y - from.y) * t;
        run_xdotool(&[
            "mousemove",
            &(x as i32).to_string(),
            &(y as i32).to_string(),
        ])?;
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    std::thread::sleep(std::time::Duration::from_millis(50));
    mouse_up(MouseButton::Left)
}

pub fn screenshot(path: &str, app: Option<&str>) -> Result<String> {
    if let Some(app_name) = app {
        let output = Command::new("xdotool")
            .args(["search", "--name", app_name])
            .output()
            .map_err(|e| Error::PlatformError {
                message: format!("xdotool not found: {e}"),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(wid) = stdout.lines().next() {
            let result = Command::new("import").args(["-window", wid, path]).output();

            if let Ok(out) = result {
                if out.status.success() {
                    return Ok(path.to_string());
                }
            }
        }
    }

    let result = Command::new("import")
        .args(["-window", "root", path])
        .output()
        .or_else(|_| Command::new("scrot").arg(path).output())
        .map_err(|e| Error::PlatformError {
            message: format!("screenshot tools not found (tried import, scrot): {e}"),
        })?;

    if result.status.success() {
        Ok(path.to_string())
    } else {
        Err(Error::PlatformError {
            message: format!(
                "screenshot failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
        })
    }
}

pub fn activate_window(app_name: &str) -> Result<()> {
    let output = Command::new("xdotool")
        .args(["search", "--name", app_name])
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool not found: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(wid) = stdout.lines().next() {
        run_xdotool(&["windowactivate", "--sync", wid])?;
        Ok(())
    } else {
        Err(Error::ApplicationNotFound {
            name: app_name.to_string(),
        })
    }
}

pub fn get_window_geometry(app_name: &str) -> Result<Option<(f64, f64, f64, f64)>> {
    let output = Command::new("xdotool")
        .args(["search", "--name", app_name])
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool not found: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let wid = match stdout.lines().next() {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };

    let geo = Command::new("xdotool")
        .args(["getwindowgeometry", "--shell", &wid])
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool getwindowgeometry failed: {e}"),
        })?;

    let geo_str = String::from_utf8_lossy(&geo.stdout);
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut w = 0.0_f64;
    let mut h = 0.0_f64;

    for line in geo_str.lines() {
        if let Some(val) = line.strip_prefix("X=") {
            x = val.parse().unwrap_or(0.0);
        } else if let Some(val) = line.strip_prefix("Y=") {
            y = val.parse().unwrap_or(0.0);
        } else if let Some(val) = line.strip_prefix("WIDTH=") {
            w = val.parse().unwrap_or(0.0);
        } else if let Some(val) = line.strip_prefix("HEIGHT=") {
            h = val.parse().unwrap_or(0.0);
        }
    }

    Ok(Some((x, y, w, h)))
}

pub fn move_window(app_name: &str, x: f64, y: f64) -> Result<bool> {
    let output = Command::new("xdotool")
        .args(["search", "--name", app_name])
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool not found: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    match stdout.lines().next() {
        Some(wid) => {
            run_xdotool(&[
                "windowmove",
                "--sync",
                wid,
                &(x as i32).to_string(),
                &(y as i32).to_string(),
            ])?;
            Ok(true)
        }
        None => Ok(false),
    }
}

pub fn resize_window(app_name: &str, width: f64, height: f64) -> Result<bool> {
    let output = Command::new("xdotool")
        .args(["search", "--name", app_name])
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool not found: {e}"),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    match stdout.lines().next() {
        Some(wid) => {
            run_xdotool(&[
                "windowsize",
                "--sync",
                wid,
                &(width as u32).to_string(),
                &(height as u32).to_string(),
            ])?;
            Ok(true)
        }
        None => Ok(false),
    }
}

fn run_xdotool(args: &[&str]) -> Result<()> {
    let output = Command::new("xdotool")
        .args(args)
        .output()
        .map_err(|e| Error::PlatformError {
            message: format!("xdotool not found — install with: sudo apt install xdotool: {e}"),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::PlatformError {
            message: format!(
                "xdotool {} failed: {}",
                args.first().unwrap_or(&""),
                String::from_utf8_lossy(&output.stderr)
            ),
        })
    }
}

fn translate_key(key_expr: &str) -> String {
    key_expr
        .split('+')
        .map(|part| {
            match part.to_lowercase().as_str() {
                "cmd" | "ctrl" | "control" => "ctrl",
                "alt" | "option" => "alt",
                "shift" => "shift",
                "super" | "meta" | "win" => "super",
                "return" | "enter" => "Return",
                "escape" | "esc" => "Escape",
                "space" => "space",
                "tab" => "Tab",
                "backspace" => "BackSpace",
                "delete" => "BackSpace",
                "del" | "forwarddelete" => "Delete",
                "up" => "Up",
                "down" => "Down",
                "left" => "Left",
                "right" => "Right",
                "home" => "Home",
                "end" => "End",
                "pageup" => "Prior",
                "pagedown" => "Next",
                "f1" => "F1",
                "f2" => "F2",
                "f3" => "F3",
                "f4" => "F4",
                "f5" => "F5",
                "f6" => "F6",
                "f7" => "F7",
                "f8" => "F8",
                "f9" => "F9",
                "f10" => "F10",
                "f11" => "F11",
                "f12" => "F12",
                other => return other.to_string(),
            }
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("+")
}
