use agent_click_core::action::{MouseButton, ScrollDirection};
use agent_click_core::node::Point;
use agent_click_core::{Error, Result};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const MOUSE_MOVE_DELAY: std::time::Duration = std::time::Duration::from_millis(10);
const MULTI_CLICK_DELAY: std::time::Duration = std::time::Duration::from_millis(50);
const CHAR_DELAY: std::time::Duration = std::time::Duration::from_millis(15);

fn screen_dimensions() -> (i32, i32) {
    unsafe {
        let cx = GetSystemMetrics(SM_CXSCREEN);
        let cy = GetSystemMetrics(SM_CYSCREEN);
        (cx, cy)
    }
}

fn to_absolute(x: f64, y: f64) -> (i32, i32) {
    let (cx, cy) = screen_dimensions();
    let abs_x = ((x * 65535.0) / cx as f64) as i32;
    let abs_y = ((y * 65535.0) / cy as f64) as i32;
    (abs_x, abs_y)
}

pub fn move_mouse(point: Point) -> Result<()> {
    let (abs_x, abs_y) = to_absolute(point.x, point.y);
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: abs_x,
                dy: abs_y,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                ..Default::default()
            },
        },
    };
    send_inputs(&[input])
}

pub fn click(point: Point, button: MouseButton, count: u32) -> Result<()> {
    move_mouse(point)?;
    std::thread::sleep(MOUSE_MOVE_DELAY);

    let (down_flag, up_flag) = match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };

    for i in 0..count {
        if i > 0 {
            std::thread::sleep(MULTI_CLICK_DELAY);
        }

        let down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: down_flag,
                    ..Default::default()
                },
            },
        };
        let up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: up_flag,
                    ..Default::default()
                },
            },
        };
        send_inputs(&[down, up])?;
    }
    Ok(())
}

pub fn scroll(direction: ScrollDirection, amount: u32) -> Result<()> {
    let (wheel_delta, horizontal) = match direction {
        ScrollDirection::Up => (120 * amount as i32, false),
        ScrollDirection::Down => (-(120 * amount as i32), false),
        ScrollDirection::Left => (120 * amount as i32, true),
        ScrollDirection::Right => (-(120 * amount as i32), true),
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                mouseData: wheel_delta as u32,
                dwFlags: if horizontal {
                    MOUSEEVENTF_HWHEEL
                } else {
                    MOUSEEVENTF_WHEEL
                },
                ..Default::default()
            },
        },
    };
    send_inputs(&[input])
}

pub fn type_text(text: &str) -> Result<()> {
    for ch in text.chars() {
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        ..Default::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        ..Default::default()
                    },
                },
            },
        ];
        send_inputs(&inputs)?;
        std::thread::sleep(CHAR_DELAY);
    }
    Ok(())
}

pub fn key_press(key_expr: &str) -> Result<()> {
    let parts: Vec<&str> = key_expr.split('+').collect();
    let (key_name, modifiers) = parts.split_last().ok_or_else(|| Error::PlatformError {
        message: format!("invalid key expression: '{key_expr}'"),
    })?;

    let mut mod_vks = Vec::new();
    for m in modifiers {
        let vk = match m.to_lowercase().as_str() {
            "cmd" | "ctrl" | "control" => VK_CONTROL,
            "alt" | "option" => VK_MENU,
            "shift" => VK_SHIFT,
            "win" | "super" | "meta" => VK_LWIN,
            _ => {
                return Err(Error::PlatformError {
                    message: format!("unknown modifier: '{m}'"),
                })
            }
        };
        mod_vks.push(vk);
    }

    let key_vk = key_name_to_vk(key_name)?;

    let mut inputs = Vec::new();

    for &vk in &mod_vks {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    ..Default::default()
                },
            },
        });
    }

    inputs.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key_vk,
                ..Default::default()
            },
        },
    });

    inputs.push(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key_vk,
                dwFlags: KEYEVENTF_KEYUP,
                ..Default::default()
            },
        },
    });

    for &vk in mod_vks.iter().rev() {
        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: KEYEVENTF_KEYUP,
                    ..Default::default()
                },
            },
        });
    }

    send_inputs(&inputs)
}

fn send_inputs(inputs: &[INPUT]) -> Result<()> {
    unsafe {
        let sent = SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        if sent as usize != inputs.len() {
            return Err(Error::PlatformError {
                message: format!("SendInput: sent {sent}/{} events", inputs.len()),
            });
        }
    }
    Ok(())
}

fn key_name_to_vk(name: &str) -> Result<VIRTUAL_KEY> {
    let vk = match name.to_lowercase().as_str() {
        "return" | "enter" => VK_RETURN,
        "tab" => VK_TAB,
        "space" => VK_SPACE,
        "backspace" | "delete" => VK_BACK,
        "escape" | "esc" => VK_ESCAPE,
        "up" => VK_UP,
        "down" => VK_DOWN,
        "left" => VK_LEFT,
        "right" => VK_RIGHT,
        "home" => VK_HOME,
        "end" => VK_END,
        "pageup" => VK_PRIOR,
        "pagedown" => VK_NEXT,
        "del" | "forwarddelete" => VK_DELETE,
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "f12" => VK_F12,
        "a" => VK_A,
        "b" => VK_B,
        "c" => VK_C,
        "d" => VK_D,
        "e" => VK_E,
        "f" => VK_F,
        "g" => VK_G,
        "h" => VK_H,
        "i" => VK_I,
        "j" => VK_J,
        "k" => VK_K,
        "l" => VK_L,
        "m" => VK_M,
        "n" => VK_N,
        "o" => VK_O,
        "p" => VK_P,
        "q" => VK_Q,
        "r" => VK_R,
        "s" => VK_S,
        "t" => VK_T,
        "u" => VK_U,
        "v" => VK_V,
        "w" => VK_W,
        "x" => VK_X,
        "y" => VK_Y,
        "z" => VK_Z,
        "0" => VK_0,
        "1" => VK_1,
        "2" => VK_2,
        "3" => VK_3,
        "4" => VK_4,
        "5" => VK_5,
        "6" => VK_6,
        "7" => VK_7,
        "8" => VK_8,
        "9" => VK_9,
        _ => {
            return Err(Error::PlatformError {
                message: format!("unknown key: '{name}'"),
            })
        }
    };
    Ok(vk)
}
