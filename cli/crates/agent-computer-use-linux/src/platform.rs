#[cfg(not(target_os = "linux"))]
mod stub {
    use agent_computer_use_core::action::{Action, ActionResult};
    use agent_computer_use_core::node::AccessibilityNode;
    use agent_computer_use_core::platform::{AppInfo, Platform, WindowInfo};
    use agent_computer_use_core::selector::Selector;
    use agent_computer_use_core::{Error, Result};
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
            platform: "Linux backend requires Linux OS".into(),
        }
    }

    #[async_trait]
    impl Platform for LinuxPlatform {
        async fn tree(
            &self,
            _app: Option<&str>,
            _max_depth: Option<u32>,
        ) -> Result<AccessibilityNode> {
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
}

#[cfg(not(target_os = "linux"))]
pub use stub::LinuxPlatform;

#[cfg(target_os = "linux")]
mod real {
    use agent_computer_use_core::action::{Action, ActionResult, MouseButton};
    use agent_computer_use_core::node::AccessibilityNode;
    use agent_computer_use_core::platform::{AppInfo, Platform, WindowInfo};
    use agent_computer_use_core::selector::Selector;
    use agent_computer_use_core::{Error, Result};
    use async_trait::async_trait;
    use tokio::sync::OnceCell;

    use crate::atspi::AtspiContext;
    use crate::input;

    pub struct LinuxPlatform {
        ctx: OnceCell<AtspiContext>,
    }

    impl LinuxPlatform {
        pub fn new() -> Self {
            Self {
                ctx: OnceCell::new(),
            }
        }

        async fn context(&self) -> Result<&AtspiContext> {
            self.ctx
                .get_or_try_init(|| async { AtspiContext::new().await })
                .await
        }
    }

    impl Default for LinuxPlatform {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl Platform for LinuxPlatform {
        async fn tree(
            &self,
            app: Option<&str>,
            max_depth: Option<u32>,
        ) -> Result<AccessibilityNode> {
            let ctx = self.context().await?;
            let (dest, path) = ctx.get_app_root(app).await?;
            Ok(ctx.element_to_node(&dest, &path, max_depth, 0).await)
        }

        async fn find(&self, selector: &Selector) -> Result<Vec<AccessibilityNode>> {
            let ctx = self.context().await?;
            let (dest, path) = ctx.get_app_root(selector.app.as_deref()).await?;
            Ok(ctx.find_all(&dest, &path, selector).await)
        }

        async fn perform(&self, action: &Action) -> Result<ActionResult> {
            match action {
                Action::Click {
                    selector,
                    coordinates,
                    button,
                    count,
                } => {
                    let point = resolve_point(self, selector.as_ref(), *coordinates).await?;
                    input::click(point, *button, *count)?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("clicked at ({}, {})", point.x, point.y)),
                        path: None,
                        data: None,
                    })
                }

                Action::Type {
                    text,
                    selector,
                    submit,
                } => {
                    if let Some(sel) = selector {
                        let node = self.find_one(sel).await?;
                        if let Some(center) = node.center() {
                            input::click(center, MouseButton::Left, 1)?;
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    }
                    input::type_text(text)?;
                    if *submit {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        input::key_press("return")?;
                    }
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("typed {} characters", text.len())),
                        path: None,
                        data: None,
                    })
                }

                Action::KeyPress { key, app } => {
                    if let Some(app_name) = app {
                        self.activate(app_name).await?;
                    }
                    input::key_press(key)?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("pressed {key}")),
                        path: None,
                        data: None,
                    })
                }

                Action::Scroll {
                    direction,
                    amount,
                    selector: _,
                    app,
                } => {
                    if let Some(app_name) = app {
                        self.activate(app_name).await?;
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    input::scroll(*direction, *amount)?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("scrolled {direction:?} by {amount}")),
                        path: None,
                        data: None,
                    })
                }

                Action::MoveMouse {
                    selector,
                    coordinates,
                } => {
                    let point = resolve_point(self, selector.as_ref(), *coordinates).await?;
                    input::move_mouse(point)?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("moved to ({}, {})", point.x, point.y)),
                        path: None,
                        data: None,
                    })
                }

                Action::Drag { from, to } => {
                    input::drag(*from, *to)?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!(
                            "dragged from ({},{}) to ({},{})",
                            from.x, from.y, to.x, to.y
                        )),
                        path: None,
                        data: None,
                    })
                }

                Action::Focus { selector } => {
                    let node = self.find_one(selector).await?;
                    if let Some(center) = node.center() {
                        input::click(center, MouseButton::Left, 1)?;
                    }
                    Ok(ActionResult {
                        success: true,
                        message: Some("focused".into()),
                        path: None,
                        data: None,
                    })
                }

                Action::Screenshot { path, app } => {
                    let save_path = path.clone().unwrap_or_else(|| {
                        format!(
                            "{}/agent-computer-use-screenshot.png",
                            std::env::temp_dir().display()
                        )
                    });
                    let saved = input::screenshot(&save_path, app.as_deref())?;
                    Ok(ActionResult {
                        success: true,
                        message: Some(format!("screenshot saved to {saved}")),
                        path: Some(saved),
                        data: None,
                    })
                }
            }
        }

        async fn focused(&self) -> Result<AccessibilityNode> {
            Err(Error::PlatformError {
                message: "focused element tracking requires AT-SPI2 event listener".into(),
            })
        }

        async fn applications(&self) -> Result<Vec<AppInfo>> {
            let ctx = self.context().await?;
            let raw = ctx.applications().await?;
            Ok(raw
                .into_iter()
                .map(|(name, _dest, pid)| AppInfo {
                    name,
                    pid,
                    frontmost: false,
                    bundle_id: None,
                })
                .collect())
        }

        async fn windows(&self, app: Option<&str>) -> Result<Vec<WindowInfo>> {
            let ctx = self.context().await?;
            let (dest, path) = ctx.get_app_root(app).await?;
            let root = ctx.element_to_node(&dest, &path, Some(2), 0).await;
            let mut windows = Vec::new();

            for child in &root.children {
                if matches!(
                    child.role,
                    agent_computer_use_core::node::Role::Window
                        | agent_computer_use_core::node::Role::Dialog
                ) {
                    windows.push(WindowInfo {
                        title: child.name.clone().unwrap_or_default(),
                        app: root.name.clone().unwrap_or_default(),
                        pid: child.pid.unwrap_or(0),
                        position: child.position,
                        size: child.size,
                        minimized: None,
                        frontmost: None,
                    });
                }
            }

            Ok(windows)
        }

        async fn text(&self, app: Option<&str>) -> Result<String> {
            let ctx = self.context().await?;
            let (dest, path) = ctx.get_app_root(app).await?;
            Ok(ctx.collect_text(&dest, &path, Some(20)).await)
        }

        async fn check_permissions(&self) -> Result<bool> {
            self.context().await?;
            Ok(true)
        }

        async fn activate(&self, app: &str) -> Result<()> {
            input::activate_window(app)?;
            Ok(())
        }

        async fn open_application(&self, app: &str) -> Result<()> {
            let result = std::process::Command::new("xdg-open")
                .arg(app)
                .spawn()
                .or_else(|_| std::process::Command::new(app).spawn());

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::PlatformError {
                    message: format!("failed to open '{app}': {e}"),
                }),
            }
        }

        async fn move_window(&self, app: &str, x: f64, y: f64) -> Result<bool> {
            input::move_window(app, x, y)
        }

        async fn resize_window(&self, app: &str, width: f64, height: f64) -> Result<bool> {
            input::resize_window(app, width, height)
        }

        fn platform_name(&self) -> &'static str {
            "Linux"
        }
    }

    async fn resolve_point(
        platform: &LinuxPlatform,
        selector: Option<&Selector>,
        coordinates: Option<agent_computer_use_core::node::Point>,
    ) -> Result<agent_computer_use_core::node::Point> {
        match (selector, coordinates) {
            (_, Some(coords)) => Ok(coords),
            (Some(sel), None) => {
                let node = platform.find_one(sel).await?;
                node.center().ok_or_else(|| Error::PlatformError {
                    message: "element has no position".into(),
                })
            }
            (None, None) => Err(Error::PlatformError {
                message: "requires selector or coordinates".into(),
            }),
        }
    }
}

#[cfg(target_os = "linux")]
pub use real::LinuxPlatform;
