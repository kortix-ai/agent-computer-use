use agent_computer_use_core::node::{AccessibilityNode, Point, Role, Size};
use agent_computer_use_core::selector::Selector;
use agent_computer_use_core::{Error, Result};
use std::collections::VecDeque;
use std::str::FromStr;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

const REGISTRY_DEST: &str = "org.a11y.atspi.Registry";
const REGISTRY_PATH: &str = "/org/a11y/atspi/accessible/root";
const COORD_TYPE_SCREEN: u32 = 0;

const STATE_BIT_ENABLED: u32 = 8;
const STATE_BIT_FOCUSED: u32 = 12;

#[zbus::proxy(
    interface = "org.a11y.atspi.Accessible",
    default_service = "org.a11y.atspi.Registry",
    default_path = "/org/a11y/atspi/accessible/root"
)]
trait Accessible {
    fn get_role(&self) -> zbus::Result<u32>;
    fn get_child_at_index(&self, index: i32) -> zbus::Result<(String, OwnedObjectPath)>;
    fn get_state(&self) -> zbus::Result<Vec<u32>>;
    fn get_application(&self) -> zbus::Result<(String, OwnedObjectPath)>;

    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn description(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn child_count(&self) -> zbus::Result<i32>;

    #[zbus(property, name = "AccessibleId")]
    fn accessible_id(&self) -> zbus::Result<String>;
}

#[zbus::proxy(
    interface = "org.a11y.atspi.Component",
    default_service = "org.a11y.atspi.Registry",
    default_path = "/org/a11y/atspi/accessible/root"
)]
trait Component {
    fn get_extents(&self, coord_type: u32) -> zbus::Result<(i32, i32, i32, i32)>;
}

#[zbus::proxy(
    interface = "org.a11y.atspi.Value",
    default_service = "org.a11y.atspi.Registry",
    default_path = "/org/a11y/atspi/accessible/root"
)]
trait Value {
    #[zbus(property)]
    fn current_value(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn set_current_value(&self, value: f64) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.a11y.atspi.Text",
    default_service = "org.a11y.atspi.Registry",
    default_path = "/org/a11y/atspi/accessible/root"
)]
trait Text {
    fn get_text(&self, start_offset: i32, end_offset: i32) -> zbus::Result<String>;
    fn get_character_count(&self) -> zbus::Result<i32>;
}

pub struct AtspiContext {
    connection: Connection,
}

impl AtspiContext {
    pub async fn new() -> Result<Self> {
        let session = Connection::session()
            .await
            .map_err(|e| plat_err(&format!("failed to connect to session bus: {e}")))?;

        let reply = session
            .call_method(
                Some("org.a11y.Bus"),
                "/org/a11y/bus",
                Some("org.a11y.Bus"),
                "GetAddress",
                &(),
            )
            .await
            .map_err(|e| plat_err(&format!("AT-SPI2 bus not available: {e}")))?;

        let address: String = reply
            .body()
            .deserialize()
            .map_err(|e| plat_err(&format!("failed to read bus address: {e}")))?;

        let address = zbus::Address::from_str(&address)
            .map_err(|e| plat_err(&format!("invalid bus address: {e}")))?;

        let connection = zbus::connection::Builder::address(address)
            .map_err(|e| plat_err(&format!("failed to create connection builder: {e}")))?
            .build()
            .await
            .map_err(|e| plat_err(&format!("failed to connect to AT-SPI2 bus: {e}")))?;

        Ok(Self { connection })
    }

    async fn accessible<'a>(&'a self, dest: &str, path: &str) -> Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(&self.connection)
            .destination(dest.to_string())
            .map_err(|e| plat_err(&format!("invalid destination '{dest}': {e}")))?
            .path(path.to_string())
            .map_err(|e| plat_err(&format!("invalid path '{path}': {e}")))?
            .build()
            .await
            .map_err(|e| plat_err(&format!("failed to build accessible proxy: {e}")))
    }

    async fn component<'a>(&'a self, dest: &str, path: &str) -> Result<ComponentProxy<'a>> {
        ComponentProxy::builder(&self.connection)
            .destination(dest.to_string())
            .map_err(|e| plat_err(&format!("invalid destination: {e}")))?
            .path(path.to_string())
            .map_err(|e| plat_err(&format!("invalid path: {e}")))?
            .build()
            .await
            .map_err(|e| plat_err(&format!("failed to build component proxy: {e}")))
    }

    async fn root(&self) -> Result<AccessibleProxy<'_>> {
        self.accessible(REGISTRY_DEST, REGISTRY_PATH).await
    }

    pub async fn find_app(&self, name: &str) -> Result<(String, String)> {
        let root = self.root().await?;
        let count = root.child_count().await.unwrap_or(0);
        let name_lower = name.to_lowercase();

        for i in 0..count {
            if let Ok((dest, path)) = root.get_child_at_index(i).await {
                let path_str = path.to_string();
                if let Ok(proxy) = self.accessible(&dest, &path_str).await {
                    if let Ok(app_name) = proxy.name().await {
                        if app_name.to_lowercase().contains(&name_lower) {
                            return Ok((dest, path_str));
                        }
                    }
                }
            }
        }

        Err(Error::ApplicationNotFound {
            name: name.to_string(),
        })
    }

    pub async fn get_app_root(&self, app: Option<&str>) -> Result<(String, String)> {
        match app {
            Some(name) => self.find_app(name).await,
            None => Ok((REGISTRY_DEST.to_string(), REGISTRY_PATH.to_string())),
        }
    }

    pub async fn element_to_node(
        &self,
        dest: &str,
        path: &str,
        max_depth: Option<u32>,
        current_depth: u32,
    ) -> AccessibilityNode {
        let proxy = match self.accessible(dest, path).await {
            Ok(p) => p,
            Err(_) => return empty_node(),
        };

        let role = proxy
            .get_role()
            .await
            .map(map_role)
            .unwrap_or(Role::Unknown);
        let name = proxy.name().await.ok().filter(|s| !s.is_empty());
        let description = proxy.description().await.ok().filter(|s| !s.is_empty());
        let id = proxy.accessible_id().await.ok().filter(|s| !s.is_empty());
        let value = self.get_value(dest, path).await;
        let (position, size) = self.get_bounds(dest, path).await;
        let (focused, enabled) = get_state_flags(&proxy).await;
        let pid = get_pid(&proxy).await;

        let children = if max_depth.is_none_or(|max| current_depth < max) {
            self.build_children(&proxy, max_depth, current_depth).await
        } else {
            vec![]
        };

        AccessibilityNode {
            role,
            name,
            value,
            description,
            id,
            position,
            size,
            focused,
            enabled,
            pid,
            children,
        }
    }

    async fn build_children(
        &self,
        proxy: &AccessibleProxy<'_>,
        max_depth: Option<u32>,
        current_depth: u32,
    ) -> Vec<AccessibilityNode> {
        let count = proxy.child_count().await.unwrap_or(0);
        let mut nodes = Vec::new();

        for i in 0..count {
            if let Ok((child_dest, child_path)) = proxy.get_child_at_index(i).await {
                let path_str = child_path.to_string();
                let node = Box::pin(self.element_to_node(
                    &child_dest,
                    &path_str,
                    max_depth,
                    current_depth + 1,
                ))
                .await;
                nodes.push(node);
            }
        }

        nodes
    }

    async fn get_value(&self, dest: &str, path: &str) -> Option<String> {
        if let Ok(proxy) = ValueProxy::builder(&self.connection)
            .destination(dest.to_string())
            .ok()?
            .path(path.to_string())
            .ok()?
            .build()
            .await
        {
            if let Ok(val) = proxy.current_value().await {
                return Some(val.to_string());
            }
        }

        if let Ok(proxy) = TextProxy::builder(&self.connection)
            .destination(dest.to_string())
            .ok()?
            .path(path.to_string())
            .ok()?
            .build()
            .await
        {
            let count = proxy.get_character_count().await.ok()?;
            if count > 0 {
                let text = proxy.get_text(0, count).await.ok()?;
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        None
    }

    async fn get_bounds(&self, dest: &str, path: &str) -> (Option<Point>, Option<Size>) {
        let proxy = match self.component(dest, path).await {
            Ok(p) => p,
            Err(_) => return (None, None),
        };

        match proxy.get_extents(COORD_TYPE_SCREEN).await {
            Ok((x, y, w, h)) if w > 0 && h > 0 => (
                Some(Point {
                    x: x as f64,
                    y: y as f64,
                }),
                Some(Size {
                    width: w as f64,
                    height: h as f64,
                }),
            ),
            _ => (None, None),
        }
    }

    pub async fn find_all(
        &self,
        root_dest: &str,
        root_path: &str,
        selector: &Selector,
    ) -> Vec<AccessibilityNode> {
        let mut results = Vec::new();
        let mut queue: VecDeque<(String, String, u32)> = VecDeque::new();
        queue.push_back((root_dest.to_string(), root_path.to_string(), 0));

        while let Some((dest, path, depth)) = queue.pop_front() {
            let proxy = match self.accessible(&dest, &path).await {
                Ok(p) => p,
                Err(_) => continue,
            };

            let node = self.shallow_node(&proxy, &dest, &path).await;
            if selector.matches(&node) {
                results.push(node);
            }

            let max = selector.max_depth.unwrap_or(u32::MAX);
            if depth < max {
                let count = proxy.child_count().await.unwrap_or(0);
                for i in 0..count {
                    if let Ok((cd, cp)) = proxy.get_child_at_index(i).await {
                        queue.push_back((cd, cp.to_string(), depth + 1));
                    }
                }
            }
        }

        results
    }

    async fn shallow_node(
        &self,
        proxy: &AccessibleProxy<'_>,
        dest: &str,
        path: &str,
    ) -> AccessibilityNode {
        let role = proxy
            .get_role()
            .await
            .map(map_role)
            .unwrap_or(Role::Unknown);
        let name = proxy.name().await.ok().filter(|s| !s.is_empty());
        let description = proxy.description().await.ok().filter(|s| !s.is_empty());
        let id = proxy.accessible_id().await.ok().filter(|s| !s.is_empty());
        let (position, size) = self.get_bounds(dest, path).await;

        AccessibilityNode {
            role,
            name,
            value: None,
            description,
            id,
            position,
            size,
            focused: None,
            enabled: None,
            pid: None,
            children: vec![],
        }
    }

    pub async fn collect_text(&self, dest: &str, path: &str, max_depth: Option<u32>) -> String {
        let mut parts = Vec::new();
        let mut stack = vec![(dest.to_string(), path.to_string(), 0u32)];

        while let Some((d, p, depth)) = stack.pop() {
            let proxy = match self.accessible(&d, &p).await {
                Ok(p) => p,
                Err(_) => continue,
            };

            let role_id = proxy.get_role().await.unwrap_or(0);
            let role = map_role(role_id);

            if matches!(role, Role::StaticText | Role::Heading | Role::Paragraph) {
                if let Ok(name) = proxy.name().await {
                    if !name.is_empty() {
                        parts.push(name);
                    }
                }
            }

            if let Some(text) = self.get_value(&d, &p).await {
                if !text.is_empty()
                    && !matches!(role, Role::StaticText | Role::Heading | Role::Paragraph)
                {
                    parts.push(text);
                }
            }

            if max_depth.is_none_or(|max| depth < max) {
                let count = proxy.child_count().await.unwrap_or(0);
                for i in (0..count).rev() {
                    if let Ok((cd, cp)) = proxy.get_child_at_index(i).await {
                        stack.push((cd, cp.to_string(), depth + 1));
                    }
                }
            }
        }

        parts.join("\n")
    }

    pub async fn applications(&self) -> Result<Vec<(String, String, u32)>> {
        let root = self.root().await?;
        let count = root.child_count().await.unwrap_or(0);
        let mut apps = Vec::new();

        for i in 0..count {
            if let Ok((dest, path)) = root.get_child_at_index(i).await {
                let path_str = path.to_string();
                if let Ok(proxy) = self.accessible(&dest, &path_str).await {
                    let name = proxy.name().await.unwrap_or_default();
                    let pid = get_pid(&proxy).await.unwrap_or(0);
                    if !name.is_empty() {
                        apps.push((name, dest, pid));
                    }
                }
            }
        }

        Ok(apps)
    }
}

async fn get_state_flags(proxy: &AccessibleProxy<'_>) -> (Option<bool>, Option<bool>) {
    match proxy.get_state().await {
        Ok(states) if !states.is_empty() => {
            let lo = states[0];
            let focused = Some(lo & (1 << STATE_BIT_FOCUSED) != 0);
            let enabled = Some(lo & (1 << STATE_BIT_ENABLED) != 0);
            (focused, enabled)
        }
        _ => (None, None),
    }
}

async fn get_pid(proxy: &AccessibleProxy<'_>) -> Option<u32> {
    let (dest, path) = proxy.get_application().await.ok()?;
    let conn = proxy.inner().connection();
    let app = AccessibleProxy::builder(conn)
        .destination(dest)
        .ok()?
        .path(path)
        .ok()?
        .build()
        .await
        .ok()?;

    let reply = conn
        .call_method(
            Some(app.inner().destination()),
            app.inner().path(),
            Some("org.a11y.atspi.Application"),
            "GetProcessId",
            &(),
        )
        .await
        .ok()?;

    reply.body().deserialize::<u32>().ok()
}

fn plat_err(message: &str) -> Error {
    Error::PlatformError {
        message: message.to_string(),
    }
}

fn empty_node() -> AccessibilityNode {
    AccessibilityNode {
        role: Role::Unknown,
        name: None,
        value: None,
        description: None,
        id: None,
        position: None,
        size: None,
        focused: None,
        enabled: None,
        pid: None,
        children: vec![],
    }
}

pub fn map_role(id: u32) -> Role {
    match id {
        7 => Role::CheckBox,
        11 => Role::ComboBox,
        16 => Role::Dialog,
        23 => Role::Window,
        26 => Role::Icon,
        27 => Role::Image,
        29 => Role::StaticText,
        31 => Role::List,
        32 => Role::ListItem,
        33 => Role::Menu,
        34 => Role::MenuBar,
        35 => Role::MenuItem,
        37 => Role::Tab,
        38 => Role::TabGroup,
        39 => Role::Group,
        40 => Role::SecureTextField,
        42 => Role::ProgressIndicator,
        43 => Role::Button,
        44 => Role::RadioButton,
        48 => Role::ScrollArea,
        49 => Role::ScrollArea,
        51 => Role::Slider,
        52 => Role::Stepper,
        53 => Role::SplitGroup,
        54 => Role::Group,
        55 => Role::Table,
        56 => Role::TableRow,
        57 => Role::TableColumn,
        61 => Role::TextField,
        62 => Role::Switch,
        63 => Role::Toolbar,
        64 => Role::Popover,
        65 => Role::Tree,
        66 => Role::Tree,
        67 => Role::Unknown,
        69 => Role::Window,
        73 => Role::Paragraph,
        75 => Role::Application,
        79 => Role::TextField,
        82 => Role::WebArea,
        83 => Role::Heading,
        85 => Role::Group,
        87 => Role::Form,
        88 => Role::Link,
        90 => Role::TableRow,
        91 => Role::TreeItem,
        95 => Role::WebArea,
        103 => Role::LevelIndicator,
        105 => Role::BlockQuote,
        109 => Role::Article,
        116 => Role::StaticText,
        129 => Role::MenuButton,
        _ => Role::Unknown,
    }
}
