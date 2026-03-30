use agent_click_core::action::{Action, MouseButton};
use agent_click_core::selector::{Selector, SelectorChain};
use agent_click_core::Platform;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::actions;
use crate::wait;

#[derive(Debug, Deserialize)]
pub struct Workflow {
    #[serde(default)]
    pub app: Option<String>,

    #[serde(default = "default_timeout")]
    pub timeout: f64,

    pub steps: Vec<Step>,
}

fn default_timeout() -> f64 {
    5.0
}

#[derive(Debug, Deserialize)]
pub struct Step {
    #[serde(default)]
    pub click: Option<String>,

    #[serde(default, rename = "type")]
    pub type_step: Option<TypeStep>,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(default)]
    pub scroll: Option<ScrollStep>,

    #[serde(default, rename = "wait-for")]
    pub wait_for: Option<String>,

    #[serde(default)]
    pub open: Option<OpenStep>,

    #[serde(default, rename = "ensure-text")]
    pub ensure_text: Option<EnsureTextStep>,

    #[serde(default)]
    pub app: Option<String>,

    #[serde(default)]
    pub timeout: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TypeStep {
    WithSelector {
        selector: String,
        text: String,
        #[serde(default)]
        submit: bool,
    },
    Simple(String),
}

#[derive(Debug, Deserialize)]
pub struct ScrollStep {
    pub direction: String,
    #[serde(default)]
    pub amount: Option<u32>,
    #[serde(default)]
    pub at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpenStep {
    WithWait {
        app: String,
        #[serde(default)]
        wait: bool,
    },
    Simple(String),
}

#[derive(Debug, Deserialize)]
pub struct EnsureTextStep {
    pub selector: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub step: usize,
    pub action: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct WorkflowError {
    pub step: usize,
    pub description: String,
    pub source: agent_click_core::Error,
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "step {}: {} — {}",
            self.step, self.description, self.source
        )
    }
}

pub async fn execute(
    platform: &dyn Platform,
    workflow: &Workflow,
    cli_app: Option<&str>,
    cli_timeout: Duration,
) -> Result<Vec<StepResult>, WorkflowError> {
    let default_app = workflow.app.as_deref().or(cli_app);
    let mut results = Vec::new();

    for (i, step) in workflow.steps.iter().enumerate() {
        let step_num = i + 1;
        let step_app = step.app.as_deref().or(default_app);
        let step_timeout = step
            .timeout
            .map(Duration::from_secs_f64)
            .unwrap_or_else(|| Duration::from_secs_f64(workflow.timeout).min(cli_timeout));

        let (action_name, message) = execute_step(platform, step, step_app, step_timeout)
            .await
            .map_err(|source| WorkflowError {
                step: step_num,
                description: describe_step(step),
                source,
            })?;

        results.push(StepResult {
            step: step_num,
            action: action_name,
            success: true,
            message,
        });
    }

    Ok(results)
}

async fn execute_step(
    platform: &dyn Platform,
    step: &Step,
    app: Option<&str>,
    timeout: Duration,
) -> agent_click_core::Result<(String, Option<String>)> {
    if let Some(ref dsl) = step.click {
        let chain = actions::parse_selector_with_app(dsl, app)?;
        let result = actions::click(platform, &chain, MouseButton::Left, 1, timeout).await?;
        return Ok(("click".into(), result.message));
    }

    if let Some(ref type_step) = step.type_step {
        return match type_step {
            TypeStep::WithSelector {
                selector,
                text,
                submit,
            } => {
                let chain = actions::parse_selector_with_app(selector, app)?;
                let result = actions::type_into(platform, &chain, text, *submit, timeout).await?;
                Ok(("type".into(), result.message))
            }
            TypeStep::Simple(text) => {
                let result = platform
                    .perform(&Action::Type {
                        text: text.clone(),
                        selector: None,
                        submit: false,
                    })
                    .await?;
                Ok(("type".into(), result.message))
            }
        };
    }

    if let Some(ref key_expr) = step.key {
        let result = platform
            .perform(&Action::KeyPress {
                key: key_expr.clone(),
                app: app.map(|a| a.to_string()),
            })
            .await?;
        return Ok(("key".into(), result.message));
    }

    if let Some(ref scroll_step) = step.scroll {
        let dir = actions::parse_direction(&scroll_step.direction)?;

        if let Some(ref at_dsl) = scroll_step.at {
            let chain = actions::parse_selector_with_app(at_dsl, app)?;
            let node = actions::find_element(platform, &chain, timeout).await?;
            let center = node
                .center()
                .ok_or_else(|| agent_click_core::Error::PlatformError {
                    message: "element has no position/size".into(),
                })?;
            platform
                .perform(&Action::MoveMouse {
                    selector: None,
                    coordinates: Some(center),
                })
                .await?;
        }

        let result = platform
            .perform(&Action::Scroll {
                direction: dir,
                amount: scroll_step.amount.unwrap_or(3),
                selector: None,
                app: app.map(|a| a.to_string()),
            })
            .await?;
        return Ok(("scroll".into(), result.message));
    }

    if let Some(ref dsl) = step.wait_for {
        let chain = actions::parse_selector_with_app(dsl, app)?;
        let node =
            wait::poll_for_element(platform, &chain, timeout, actions::POLL_INTERVAL).await?;
        let msg = format!("found {:?} {:?}", node.role, node.name.unwrap_or_default());
        return Ok(("wait-for".into(), Some(msg)));
    }

    if let Some(ref open_step) = step.open {
        let (app_name, do_wait) = match open_step {
            OpenStep::WithWait { app, wait } => (app.as_str(), *wait),
            OpenStep::Simple(name) => (name.as_str(), false),
        };

        platform.open_application(app_name).await?;

        if do_wait {
            let chain = SelectorChain::single(Selector::new().with_app(app_name));
            wait::poll_for_element(platform, &chain, timeout, Duration::from_millis(500)).await?;
        }

        return Ok(("open".into(), Some(format!("opened '{app_name}'"))));
    }

    if let Some(ref ensure) = step.ensure_text {
        let chain = actions::parse_selector_with_app(&ensure.selector, app)?;
        let node = actions::find_element(platform, &chain, timeout).await?;

        if let Some(ref current_value) = node.value {
            if current_value == &ensure.text {
                return Ok(("ensure-text".into(), Some("text already matches".into())));
            }
        }

        let result = actions::type_into(platform, &chain, &ensure.text, false, timeout).await?;
        return Ok(("ensure-text".into(), result.message));
    }

    Err(agent_click_core::Error::PlatformError {
        message: "step has no action defined".into(),
    })
}

fn describe_step(step: &Step) -> String {
    if let Some(ref s) = step.click {
        return format!("click '{s}'");
    }
    if let Some(ref t) = step.type_step {
        return match t {
            TypeStep::Simple(text) => format!("type '{text}'"),
            TypeStep::WithSelector { selector, text, .. } => {
                format!("type '{text}' into '{selector}'")
            }
        };
    }
    if let Some(ref k) = step.key {
        return format!("key '{k}'");
    }
    if step.scroll.is_some() {
        return "scroll".into();
    }
    if let Some(ref s) = step.wait_for {
        return format!("wait-for '{s}'");
    }
    if let Some(ref o) = step.open {
        let name = match o {
            OpenStep::Simple(n) => n.as_str(),
            OpenStep::WithWait { app, .. } => app.as_str(),
        };
        return format!("open '{name}'");
    }
    if let Some(ref e) = step.ensure_text {
        return format!("ensure-text '{}'", e.selector);
    }
    "unknown step".into()
}
