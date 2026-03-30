use agent_click_core::action::{Action, ActionResult};
use agent_click_core::node::AccessibilityNode;
use agent_click_core::platform::{AppInfo, Platform, WindowInfo};
use agent_click_core::selector::Selector;
use agent_click_core::{Error, Result};
use async_trait::async_trait;

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

fn not_implemented() -> Error {
    Error::UnsupportedPlatform {
        platform: "Linux (not yet implemented — contributions welcome!)".into(),
    }
}

#[async_trait]
impl Platform for LinuxPlatform {
    async fn tree(&self, _app: Option<&str>, _max_depth: Option<u32>) -> Result<AccessibilityNode> {
        Err(not_implemented())
    }

    async fn find(&self, _selector: &Selector) -> Result<Vec<AccessibilityNode>> {
        Err(not_implemented())
    }

    async fn perform(&self, _action: &Action) -> Result<ActionResult> {
        Err(not_implemented())
    }

    async fn focused(&self) -> Result<AccessibilityNode> {
        Err(not_implemented())
    }

    async fn applications(&self) -> Result<Vec<AppInfo>> {
        Err(not_implemented())
    }

    async fn windows(&self, _app: Option<&str>) -> Result<Vec<WindowInfo>> {
        Err(not_implemented())
    }

    async fn text(&self, _app: Option<&str>) -> Result<String> {
        Err(not_implemented())
    }

    async fn check_permissions(&self) -> Result<bool> {
        Err(not_implemented())
    }

    fn platform_name(&self) -> &'static str {
        "Linux"
    }
}
