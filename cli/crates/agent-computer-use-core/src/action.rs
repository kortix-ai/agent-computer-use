use crate::node::Point;
use crate::selector::Selector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Click {
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,

        #[serde(skip_serializing_if = "Option::is_none")]
        coordinates: Option<Point>,

        #[serde(default)]
        button: MouseButton,

        #[serde(default = "default_click_count")]
        count: u32,
    },

    Type {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,

        #[serde(default)]
        submit: bool,
    },

    KeyPress {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },

    Scroll {
        direction: ScrollDirection,
        #[serde(default = "default_scroll_amount")]
        amount: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },

    MoveMouse {
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,
        #[serde(skip_serializing_if = "Option::is_none")]
        coordinates: Option<Point>,
    },

    Drag {
        from: Point,
        to: Point,
    },

    Focus {
        selector: Selector,
    },
    Screenshot {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

fn default_click_count() -> u32 {
    1
}

fn default_scroll_amount() -> u32 {
    3
}
