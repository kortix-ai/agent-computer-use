use agent_click_core::action::ActionResult;
use serde::Serialize;

pub struct Output {
    human: bool,
    compact: bool,
}

impl Output {
    pub fn new(human: bool, compact: bool) -> Self {
        Self { human, compact }
    }

    pub fn print<T: Serialize>(&self, value: &T) {
        let json = if self.compact {
            serde_json::to_string(value)
        } else {
            serde_json::to_string_pretty(value)
        };

        match json {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize output: {e}");
                std::process::exit(1);
            }
        }
    }

    pub fn error(&self, err: &agent_click_core::Error) {
        if self.compact || !self.human {
            let error_json = serde_json::json!({
                "error": true,
                "type": error_type_name(err),
                "message": err.to_string(),
            });
            eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap());
        } else {
            eprintln!("error: {err}");
        }
    }
}

#[derive(Serialize)]
pub struct ExpectResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub expect: ExpectOutcome,
}

#[derive(Serialize)]
pub struct ExpectOutcome {
    pub met: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<agent_click_core::AccessibilityNode>,
}

pub enum RunError {
    Core(agent_click_core::Error),
    ExpectFailed {
        action_result: ActionResult,
        message: String,
    },
}

impl From<agent_click_core::Error> for RunError {
    fn from(e: agent_click_core::Error) -> Self {
        RunError::Core(e)
    }
}

fn error_type_name(err: &agent_click_core::Error) -> &'static str {
    match err {
        agent_click_core::Error::ElementNotFound { .. } => "element_not_found",
        agent_click_core::Error::AmbiguousSelector { .. } => "ambiguous_selector",
        agent_click_core::Error::ActionNotSupported { .. } => "action_not_supported",
        agent_click_core::Error::PermissionDenied { .. } => "permission_denied",
        agent_click_core::Error::ApplicationNotFound { .. } => "application_not_found",
        agent_click_core::Error::PlatformError { .. } => "platform_error",
        agent_click_core::Error::UnsupportedPlatform { .. } => "unsupported_platform",
        agent_click_core::Error::Timeout { .. } => "timeout",
        agent_click_core::Error::Serialization(_) => "serialization",
        agent_click_core::Error::Io(_) => "io",
    }
}
