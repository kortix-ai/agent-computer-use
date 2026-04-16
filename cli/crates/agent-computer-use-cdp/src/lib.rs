pub mod connection;
pub mod detect;
pub mod dom;
pub mod protocol;

use agent_computer_use_core::action::{Action, ActionResult};
use agent_computer_use_core::node::{AccessibilityNode, Role};
use agent_computer_use_core::platform::{AppInfo, WindowInfo};
use agent_computer_use_core::selector::Selector;
use agent_computer_use_core::{Error, Platform};
use connection::CdpConnection;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct CdpConfig {
    pub port: Option<u16>,
    pub force: bool,
    pub disabled: bool,
}

pub struct ElectronAwarePlatform<P: Platform> {
    inner: P,
    config: CdpConfig,
    connections: Mutex<HashMap<String, std::sync::Arc<CdpConnection>>>,
}

impl<P: Platform> ElectronAwarePlatform<P> {
    pub fn new(inner: P, config: CdpConfig) -> Self {
        Self {
            inner,
            config,
            connections: Mutex::new(HashMap::new()),
        }
    }

    async fn get_connection(&self, app: &str) -> Option<std::sync::Arc<CdpConnection>> {
        if self.config.disabled {
            return None;
        }

        if let Ok(conns) = self.connections.lock() {
            if let Some(conn) = conns.get(app) {
                return Some(conn.clone());
            }
        }

        if !self.config.force && !detect::is_electron_app(app) {
            return None;
        }

        let port = self
            .config
            .port
            .or_else(|| detect::find_cdp_port(app))
            .or_else(|| {
                tracing::debug!("no CDP port for {app}, relaunching with CDP");
                eprintln!("agent-computer-use: relaunching {app} with CDP support...");
                detect::relaunch_with_cdp(app)
            })?;

        tracing::debug!("connecting to CDP for {app} on port {port}");

        match CdpConnection::connect(port).await {
            Ok(conn) => {
                let conn = std::sync::Arc::new(conn);
                if let Ok(mut conns) = self.connections.lock() {
                    conns.insert(app.to_string(), conn.clone());
                }
                Some(conn)
            }
            Err(e) => {
                tracing::debug!("CDP connection failed for {app}: {e}");
                None
            }
        }
    }

    fn any_connection(&self) -> Option<std::sync::Arc<CdpConnection>> {
        self.connections
            .lock()
            .ok()
            .and_then(|c| c.values().next().cloned())
    }

    async fn window_offset(&self, app: &str) -> (f64, f64) {
        if let Ok(windows) = self.inner.windows(Some(app)).await {
            if let Some(win) = windows.first() {
                if let Some(pos) = win.position {
                    return (pos.x, pos.y + 28.0);
                }
            }
        }
        (0.0, 0.0)
    }

    fn has_webarea(node: &AccessibilityNode) -> bool {
        if node.role == Role::WebArea {
            return true;
        }
        node.children.iter().any(Self::has_webarea)
    }

    fn inject_cdp_tree(
        mut native: AccessibilityNode,
        cdp_tree: AccessibilityNode,
    ) -> AccessibilityNode {
        let webarea = AccessibilityNode {
            role: Role::WebArea,
            name: Some("Web Content".into()),
            children: cdp_tree.children,
            ..cdp_tree
        };

        if Self::has_webarea(&native) {
            return Self::replace_webarea(native, webarea);
        }

        for child in &mut native.children {
            if child.role == Role::Window {
                child.children.push(webarea);
                return native;
            }
        }

        native.children.push(webarea);
        native
    }

    fn replace_webarea(
        mut node: AccessibilityNode,
        webarea: AccessibilityNode,
    ) -> AccessibilityNode {
        node.children = node
            .children
            .into_iter()
            .map(|child| {
                if child.role == Role::WebArea {
                    AccessibilityNode {
                        children: webarea.children.clone(),
                        ..child
                    }
                } else if Self::has_webarea(&child) {
                    Self::replace_webarea(child, webarea.clone())
                } else {
                    child
                }
            })
            .collect();
        node
    }
}

#[async_trait]
impl<P: Platform> Platform for ElectronAwarePlatform<P> {
    async fn tree(
        &self,
        app: Option<&str>,
        max_depth: Option<u32>,
    ) -> agent_computer_use_core::Result<AccessibilityNode> {
        if let Some(app_name) = app {
            if let Some(conn) = self.get_connection(app_name).await {
                let offset = self.window_offset(app_name).await;
                match dom::get_dom_tree(&conn, offset).await {
                    Ok(cdp_tree) => {
                        tracing::debug!("got CDP DOM tree for {app_name}");
                        let native = self.inner.tree(app, Some(5)).await?;
                        return Ok(Self::inject_cdp_tree(native, cdp_tree));
                    }
                    Err(e) => tracing::debug!("CDP tree failed: {e}"),
                }
            }
        }
        self.inner.tree(app, max_depth).await
    }

    async fn find(
        &self,
        selector: &Selector,
    ) -> agent_computer_use_core::Result<Vec<AccessibilityNode>> {
        if let Some(ref id) = selector.id {
            if let Some(tag) = dom::extract_cdp_tag(id) {
                if let Some(ref app_name) = selector.app {
                    if let Some(conn) = self.get_connection(app_name).await {
                        let offset = self.window_offset(app_name).await;
                        if let Ok(Some(node)) = dom::get_value_by_tag(&conn, tag, offset).await {
                            return Ok(vec![node]);
                        }
                    }
                }
            }
        }

        if let Some(ref css) = selector.css {
            if let Some(ref app_name) = selector.app {
                if let Some(conn) = self.get_connection(app_name).await {
                    let offset = self.window_offset(app_name).await;
                    return dom::query_selector_all(&conn, css, offset).await;
                }
            }
            return Err(Error::PlatformError {
                message: "css= selector requires CDP connection".into(),
            });
        }

        self.inner.find(selector).await
    }

    async fn perform(&self, action: &Action) -> agent_computer_use_core::Result<ActionResult> {
        if let Action::KeyPress {
            key,
            app: Some(ref app_name),
        } = action
        {
            if let Some(conn) = self.get_connection(app_name).await {
                dom::dispatch_key(&conn, key).await?;
                return Ok(ActionResult {
                    success: true,
                    message: Some(format!("pressed {key} via CDP")),
                    path: None,
                    data: None,
                });
            }
        }

        if let Action::Type {
            text,
            selector: None,
            submit,
        } = action
        {
            if let Some(conn) = self.any_connection() {
                dom::insert_text(&conn, text).await?;
                if *submit {
                    dom::dispatch_key(&conn, "Return").await?;
                }
                return Ok(ActionResult {
                    success: true,
                    message: Some(format!("typed {} characters via CDP", text.len())),
                    path: None,
                    data: None,
                });
            }
        }

        if let Action::Scroll {
            direction,
            amount,
            app: Some(ref app_name),
            selector,
        } = action
        {
            if let Some(conn) = self.get_connection(app_name).await {
                let dir = format!("{direction:?}").to_lowercase();
                let at_css = selector.as_ref().and_then(|s| s.css.as_deref());
                dom::scroll_page(&conn, &dir, *amount, at_css).await?;
                return Ok(ActionResult {
                    success: true,
                    message: Some(format!("scrolled {dir} via CDP")),
                    path: None,
                    data: None,
                });
            }
        }

        if let Action::Click {
            selector: Some(sel),
            ..
        } = action
        {
            if let Some(ref css) = sel.css {
                if let Some(ref app_name) = sel.app {
                    if let Some(conn) = self.get_connection(app_name).await {
                        let clicked = dom::click_by_css(&conn, css).await?;
                        return Ok(ActionResult {
                            success: clicked,
                            message: Some(if clicked {
                                format!("clicked via CDP: {css}")
                            } else {
                                format!("element not found: {css}")
                            }),
                            path: None,
                            data: None,
                        });
                    }
                }
            }
        }

        self.inner.perform(action).await
    }

    async fn text(&self, app: Option<&str>) -> agent_computer_use_core::Result<String> {
        if let Some(app_name) = app {
            if let Some(conn) = self.get_connection(app_name).await {
                if let Ok(text) = dom::get_page_text(&conn).await {
                    return Ok(text);
                }
            }
        }
        self.inner.text(app).await
    }

    async fn press(&self, selector: &Selector) -> agent_computer_use_core::Result<bool> {
        if let Some(ref id) = selector.id {
            if let Some(tag) = dom::extract_cdp_tag(id) {
                if let Some(ref app_name) = selector.app {
                    if let Some(conn) = self.get_connection(app_name).await {
                        tracing::debug!("pressing via CDP: data-acu={tag}");
                        return dom::click_by_tag(&conn, tag).await;
                    }
                }
            }
        }
        self.inner.press(selector).await
    }

    async fn set_value(
        &self,
        selector: &Selector,
        value: &str,
    ) -> agent_computer_use_core::Result<bool> {
        if let Some(ref id) = selector.id {
            if let Some(tag) = dom::extract_cdp_tag(id) {
                if let Some(ref app_name) = selector.app {
                    if let Some(conn) = self.get_connection(app_name).await {
                        tracing::debug!("typing via CDP: data-acu={tag}");
                        return dom::type_into_tag(&conn, tag, value).await;
                    }
                }
            }
        }
        self.inner.set_value(selector, value).await
    }

    async fn activate(&self, app: &str) -> agent_computer_use_core::Result<()> {
        if self.get_connection(app).await.is_some() {
            tracing::debug!("skipping activate for CDP app {app}");
            return Ok(());
        }
        self.inner.activate(app).await
    }

    async fn focused(&self) -> agent_computer_use_core::Result<AccessibilityNode> {
        self.inner.focused().await
    }

    async fn element_at_point(
        &self,
        x: f64,
        y: f64,
    ) -> agent_computer_use_core::Result<Option<AccessibilityNode>> {
        self.inner.element_at_point(x, y).await
    }

    async fn applications(&self) -> agent_computer_use_core::Result<Vec<AppInfo>> {
        self.inner.applications().await
    }

    async fn windows(&self, app: Option<&str>) -> agent_computer_use_core::Result<Vec<WindowInfo>> {
        self.inner.windows(app).await
    }

    async fn check_permissions(&self) -> agent_computer_use_core::Result<bool> {
        self.inner.check_permissions().await
    }

    async fn scroll_to_visible(
        &self,
        selector: &Selector,
    ) -> agent_computer_use_core::Result<bool> {
        self.inner.scroll_to_visible(selector).await
    }

    async fn open_application(&self, app: &str) -> agent_computer_use_core::Result<()> {
        self.inner.open_application(app).await
    }

    async fn move_window(
        &self,
        app: &str,
        x: f64,
        y: f64,
    ) -> agent_computer_use_core::Result<bool> {
        self.inner.move_window(app, x, y).await
    }

    async fn resize_window(
        &self,
        app: &str,
        width: f64,
        height: f64,
    ) -> agent_computer_use_core::Result<bool> {
        self.inner.resize_window(app, width, height).await
    }

    fn platform_name(&self) -> &'static str {
        "cdp-aware"
    }
}
