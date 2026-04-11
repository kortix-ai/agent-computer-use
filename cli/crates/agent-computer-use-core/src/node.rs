use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    pub role: Role,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Point>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<AccessibilityNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Application,
    Window,
    Dialog,
    Sheet,
    Popover,

    Group,
    ScrollArea,
    SplitGroup,
    TabGroup,
    Toolbar,

    Button,
    CheckBox,
    RadioButton,
    TextField,
    TextArea,
    SecureTextField,
    Slider,
    Stepper,
    Switch,
    ComboBox,
    PopUpButton,
    DisclosureTriangle,
    ColorWell,
    DatePicker,

    Menu,
    MenuBar,
    MenuItem,
    MenuButton,
    List,
    ListItem,
    Table,
    TableRow,
    TableColumn,
    Outline,
    OutlineRow,
    Tree,
    TreeItem,

    Tab,
    TabPanel,
    Link,
    NavigationBar,

    StaticText,
    Image,
    Icon,
    ProgressIndicator,
    BusyIndicator,
    LevelIndicator,

    WebArea,
    Heading,
    Paragraph,
    BlockQuote,
    Form,
    Article,
    Banner,
    Complementary,
    ContentInfo,
    Main,
    Search,

    SystemWide,
    Unknown,

    #[serde(untagged)]
    Other(String),
}

impl Role {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "application" => Role::Application,
            "window" => Role::Window,
            "dialog" => Role::Dialog,
            "sheet" => Role::Sheet,
            "group" => Role::Group,
            "scrollarea" => Role::ScrollArea,
            "splitgroup" => Role::SplitGroup,
            "tabgroup" => Role::TabGroup,
            "toolbar" => Role::Toolbar,
            "button" => Role::Button,
            "checkbox" => Role::CheckBox,
            "radiobutton" => Role::RadioButton,
            "textfield" => Role::TextField,
            "textarea" => Role::TextArea,
            "securetextfield" => Role::SecureTextField,
            "slider" => Role::Slider,
            "stepper" => Role::Stepper,
            "switch" => Role::Switch,
            "combobox" => Role::ComboBox,
            "popupbutton" => Role::PopUpButton,
            "menu" => Role::Menu,
            "menubar" => Role::MenuBar,
            "menuitem" => Role::MenuItem,
            "menubutton" => Role::MenuButton,
            "list" => Role::List,
            "listitem" => Role::ListItem,
            "table" => Role::Table,
            "tablerow" => Role::TableRow,
            "outline" => Role::Outline,
            "tab" => Role::Tab,
            "link" => Role::Link,
            "statictext" | "text" => Role::StaticText,
            "image" => Role::Image,
            "progressindicator" | "progress" => Role::ProgressIndicator,
            "webarea" => Role::WebArea,
            "heading" => Role::Heading,
            "paragraph" => Role::Paragraph,
            "form" => Role::Form,
            "search" => Role::Search,
            other => Role::Other(other.to_string()),
        }
    }
}

impl AccessibilityNode {
    pub fn center(&self) -> Option<Point> {
        match (self.position, self.size) {
            (Some(pos), Some(size)) => Some(Point {
                x: pos.x + size.width / 2.0,
                y: pos.y + size.height / 2.0,
            }),
            _ => None,
        }
    }

    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    pub fn walk_path(&self, path: &[usize]) -> Option<&AccessibilityNode> {
        let mut current = self;
        for &index in path {
            current = current.children.get(index)?;
        }
        Some(current)
    }

    pub fn find_all<F>(&self, predicate: &F) -> Vec<&AccessibilityNode>
    where
        F: Fn(&AccessibilityNode) -> bool,
    {
        let mut results = Vec::new();
        if predicate(self) {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_all(predicate));
        }
        results
    }

    pub fn find_first<F>(&self, predicate: &F) -> Option<&AccessibilityNode>
    where
        F: Fn(&AccessibilityNode) -> bool,
    {
        if predicate(self) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_first(predicate) {
                return Some(found);
            }
        }
        None
    }
}
