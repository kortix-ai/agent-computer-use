use crate::node::{AccessibilityNode, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorChain {
    pub selectors: Vec<Selector>,
}

impl SelectorChain {
    pub fn single(selector: Selector) -> Self {
        Self {
            selectors: vec![selector],
        }
    }

    pub fn first(&self) -> &Selector {
        &self.selectors[0]
    }

    pub fn is_simple(&self) -> bool {
        self.selectors.len() == 1
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selector {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_contains: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<usize>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    /// CSS selector for CDP/Electron apps (e.g., `css=".my-button"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
}

impl Selector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_app(mut self, app: impl Into<String>) -> Self {
        self.app = Some(app.into());
        self
    }

    pub fn with_role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_name_contains(mut self, substring: impl Into<String>) -> Self {
        self.name_contains = Some(substring.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn with_path(mut self, path: Vec<usize>) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn matches(&self, node: &AccessibilityNode) -> bool {
        if let Some(ref role) = self.role {
            if &node.role != role {
                return false;
            }
        }

        if let Some(ref name) = self.name {
            match &node.name {
                Some(node_name) if node_name == name => {}
                _ => return false,
            }
        }

        if let Some(ref substring) = self.name_contains {
            match &node.name {
                Some(node_name) if node_name.to_lowercase().contains(&substring.to_lowercase()) => {
                }
                _ => return false,
            }
        }

        if let Some(ref id) = self.id {
            match &node.id {
                Some(node_id) if node_id == id => {}
                _ => return false,
            }
        }

        if let Some(ref substring) = self.id_contains {
            match &node.id {
                Some(node_id) if node_id.to_lowercase().contains(&substring.to_lowercase()) => {}
                _ => return false,
            }
        }

        true
    }
}
