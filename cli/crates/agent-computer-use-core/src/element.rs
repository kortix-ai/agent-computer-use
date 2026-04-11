use crate::node::{AccessibilityNode, Role};
use crate::{Error, Result};

pub fn is_interactive(role: &Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::TextField
            | Role::TextArea
            | Role::SecureTextField
            | Role::CheckBox
            | Role::RadioButton
            | Role::Link
            | Role::Tab
            | Role::Slider
            | Role::Switch
            | Role::ComboBox
            | Role::PopUpButton
            | Role::MenuItem
            | Role::MenuButton
            | Role::ListItem
            | Role::Stepper
            | Role::DisclosureTriangle
    )
}

pub fn is_visible(node: &AccessibilityNode) -> bool {
    matches!(
        (node.position, node.size),
        (Some(_), Some(size)) if size.width > 0.0 && size.height > 0.0
    )
}

pub fn check_visible(node: &AccessibilityNode) -> Result<()> {
    match (node.position, node.size) {
        (Some(pos), Some(size)) => {
            if size.width <= 0.0 || size.height <= 0.0 {
                return Err(Error::PlatformError {
                    message: format!(
                        "element {:?} has zero size ({:.0}x{:.0})",
                        node.name.as_deref().unwrap_or("(unnamed)"),
                        size.width,
                        size.height
                    ),
                });
            }
            if pos.x < 0.0 || pos.y < 0.0 {
                tracing::warn!(
                    "element {:?} may be offscreen (position: {:.0}, {:.0})",
                    node.name.as_deref().unwrap_or("(unnamed)"),
                    pos.x,
                    pos.y
                );
            }
            Ok(())
        }
        _ => Err(Error::PlatformError {
            message: format!(
                "element {:?} has no position/size — cannot target by coordinates",
                node.name.as_deref().unwrap_or("(unnamed)")
            ),
        }),
    }
}

pub fn rank(node: &AccessibilityNode) -> (i32, i32, i32) {
    let visible = if is_visible(node) { 1 } else { 0 };

    let enabled = match node.enabled {
        Some(true) | None => 1,
        Some(false) => 0,
    };

    let interactive = match node.role {
        Role::Button
        | Role::TextField
        | Role::TextArea
        | Role::SecureTextField
        | Role::CheckBox
        | Role::RadioButton
        | Role::Slider
        | Role::Switch
        | Role::ComboBox
        | Role::PopUpButton
        | Role::Link
        | Role::Tab
        | Role::Stepper
        | Role::DisclosureTriangle => 2,
        Role::MenuItem | Role::MenuButton | Role::ListItem | Role::TableRow | Role::TreeItem => 1,
        _ => 0,
    };

    (visible, enabled, interactive)
}

pub fn collect_text(node: &AccessibilityNode) -> String {
    let mut parts = Vec::new();
    collect_text_recursive(node, &mut parts);
    parts.join("\n")
}

fn collect_text_recursive(node: &AccessibilityNode, parts: &mut Vec<String>) {
    if matches!(node.role, Role::StaticText) {
        if let Some(ref name) = node.name {
            if !name.is_empty() {
                parts.push(name.clone());
            }
        }
    }
    if let Some(ref value) = node.value {
        if !value.is_empty() && !matches!(node.role, Role::StaticText) {
            parts.push(value.clone());
        }
    }
    for child in &node.children {
        collect_text_recursive(child, parts);
    }
}
