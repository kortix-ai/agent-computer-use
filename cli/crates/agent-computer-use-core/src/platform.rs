use crate::action::{Action, ActionResult};
use crate::error::Result;
use crate::node::AccessibilityNode;
use crate::selector::Selector;
use async_trait::async_trait;

#[async_trait]
pub trait Platform: Send + Sync {
    async fn tree(&self, app: Option<&str>, max_depth: Option<u32>) -> Result<AccessibilityNode>;

    async fn find(&self, selector: &Selector) -> Result<Vec<AccessibilityNode>>;

    async fn find_one(&self, selector: &Selector) -> Result<AccessibilityNode> {
        let results = self.find(selector).await?;
        match results.len() {
            0 => Err(crate::Error::ElementNotFound {
                message: format!("{selector:?}"),
            }),
            1 => Ok(results.into_iter().next().unwrap()),
            n => Err(crate::Error::AmbiguousSelector { count: n }),
        }
    }

    async fn perform(&self, action: &Action) -> Result<ActionResult>;

    async fn focused(&self) -> Result<AccessibilityNode>;

    async fn applications(&self) -> Result<Vec<AppInfo>>;

    async fn windows(&self, app: Option<&str>) -> Result<Vec<WindowInfo>>;

    async fn text(&self, app: Option<&str>) -> Result<String>;

    async fn check_permissions(&self) -> Result<bool>;

    async fn activate(&self, _app: &str) -> Result<()> {
        Ok(())
    }

    async fn press(&self, _selector: &Selector) -> Result<bool> {
        Ok(false)
    }

    async fn set_value(&self, _selector: &Selector, _value: &str) -> Result<bool> {
        Ok(false)
    }

    async fn scroll_to_visible(&self, _selector: &Selector) -> Result<bool> {
        Ok(false)
    }

    async fn open_application(&self, _app: &str) -> Result<()> {
        Err(crate::Error::UnsupportedPlatform {
            platform: "open_application not implemented".into(),
        })
    }

    async fn move_window(&self, _app: &str, _x: f64, _y: f64) -> Result<bool> {
        Ok(false)
    }

    async fn resize_window(&self, _app: &str, _width: f64, _height: f64) -> Result<bool> {
        Ok(false)
    }

    /// Resolve the element under screen-coordinate (x, y). Returns None if unsupported
    /// on the current platform or no element can be determined.
    async fn element_at_point(&self, _x: f64, _y: f64) -> Result<Option<AccessibilityNode>> {
        Ok(None)
    }

    fn platform_name(&self) -> &'static str;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub pid: u32,
    pub frontmost: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app: String,
    pub pid: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<crate::node::Point>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<crate::node::Size>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimized: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmost: Option<bool>,
}
