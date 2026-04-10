use agent_click_core::node::{AccessibilityNode, Point, Role, Size};
use agent_click_core::selector::Selector;
use agent_click_core::{Error, Result};
use std::collections::VecDeque;
use std::sync::OnceLock;
use windows::core::BSTR;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::UI::Accessibility::*;

struct SendSyncCondition(IUIAutomationCondition);
unsafe impl Send for SendSyncCondition {}
unsafe impl Sync for SendSyncCondition {}

static TRUE_CONDITION: OnceLock<SendSyncCondition> = OnceLock::new();

pub struct UiaContext {
    pub automation: IUIAutomation,
}

unsafe impl Send for UiaContext {}
unsafe impl Sync for UiaContext {}

impl UiaContext {
    pub fn new() -> Result<Self> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let automation: IUIAutomation =
                windows::Win32::System::Com::CoCreateInstance::<_, IUIAutomation>(
                    &CUIAutomation,
                    None,
                    windows::Win32::System::Com::CLSCTX_INPROC_SERVER,
                )
                .map_err(|e| Error::PlatformError {
                    message: format!("failed to create IUIAutomation: {e}"),
                })?;
            TRUE_CONDITION.get_or_init(|| {
                SendSyncCondition(
                    automation
                        .CreateTrueCondition()
                        .expect("CreateTrueCondition"),
                )
            });

            Ok(Self { automation })
        }
    }

    pub fn root(&self) -> Result<IUIAutomationElement> {
        unsafe {
            self.automation
                .GetRootElement()
                .map_err(|e| Error::PlatformError {
                    message: format!("failed to get root element: {e}"),
                })
        }
    }

    pub fn focused_element(&self) -> Result<IUIAutomationElement> {
        unsafe {
            self.automation
                .GetFocusedElement()
                .map_err(|e| Error::PlatformError {
                    message: format!("failed to get focused element: {e}"),
                })
        }
    }

    #[allow(dead_code)]
    pub fn element_for_pid(&self, pid: u32) -> Result<Option<IUIAutomationElement>> {
        unsafe {
            let root = self.root()?;
            let condition = self
                .automation
                .CreatePropertyCondition(
                    UIA_ProcessIdPropertyId,
                    &windows::core::VARIANT::from(pid as i32),
                )
                .map_err(|e| Error::PlatformError {
                    message: format!("failed to create PID condition: {e}"),
                })?;

            let found = root.FindFirst(TreeScope_Children, &condition);
            match found {
                Ok(el) => Ok(Some(el)),
                Err(_) => Ok(None),
            }
        }
    }

    pub fn find_app_element(&self, app_name: &str) -> Result<IUIAutomationElement> {
        unsafe {
            let root = self.root()?;
            let walker = self
                .automation
                .CreateTreeWalker(&self.automation.RawViewCondition().map_err(|e| {
                    Error::PlatformError {
                        message: format!("failed to create tree walker: {e}"),
                    }
                })?)
                .map_err(|e| Error::PlatformError {
                    message: format!("failed to create tree walker: {e}"),
                })?;

            let mut child = walker.GetFirstChildElement(&root).ok();
            let app_lower = app_name.to_lowercase();

            while let Some(ref el) = child {
                if let Ok(name) = el.CurrentName() {
                    if name.to_string().to_lowercase().contains(&app_lower) {
                        return Ok(el.clone());
                    }
                }
                child = walker.GetNextSiblingElement(el).ok();
            }

            Err(Error::ApplicationNotFound {
                name: app_name.to_string(),
            })
        }
    }
}

pub fn element_to_node(
    element: &IUIAutomationElement,
    max_depth: Option<u32>,
    current_depth: u32,
) -> AccessibilityNode {
    unsafe {
        let role = map_control_type(element.CurrentControlType().unwrap_or_default());
        let name = element
            .CurrentName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        let id = element
            .CurrentAutomationId()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let value = get_value(element);

        let (position, size) = get_bounds(element);

        let focused = element.CurrentHasKeyboardFocus().ok().map(|b| b.as_bool());
        let enabled = element.CurrentIsEnabled().ok().map(|b| b.as_bool());
        let pid = element.CurrentProcessId().ok().map(|p| p as u32);

        let children = if max_depth.is_none_or(|max| current_depth < max) {
            get_children(element)
                .into_iter()
                .map(|c| element_to_node(&c, max_depth, current_depth + 1))
                .collect()
        } else {
            vec![]
        };

        AccessibilityNode {
            role,
            name,
            value,
            description: None,
            id,
            position,
            size,
            focused,
            enabled,
            pid,
            children,
        }
    }
}

fn get_value(element: &IUIAutomationElement) -> Option<String> {
    unsafe {
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            if let Ok(val) = pattern.CurrentValue() {
                let s = val.to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }

        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId)
        {
            if let Ok(state) = pattern.CurrentToggleState() {
                #[allow(non_upper_case_globals)]
                let label = match state {
                    ToggleState_On => "checked",
                    ToggleState_Off => "unchecked",
                    _ => "indeterminate",
                };
                return Some(label.to_string());
            }
        }

        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationRangeValuePattern>(UIA_RangeValuePatternId)
        {
            if let Ok(val) = pattern.CurrentValue() {
                return Some(val.to_string());
            }
        }

        None
    }
}

fn get_bounds(element: &IUIAutomationElement) -> (Option<Point>, Option<Size>) {
    unsafe {
        match element.CurrentBoundingRectangle() {
            Ok(rect) => {
                let x = rect.left as f64;
                let y = rect.top as f64;
                let w = (rect.right - rect.left) as f64;
                let h = (rect.bottom - rect.top) as f64;
                if w > 0.0 && h > 0.0 {
                    (
                        Some(Point { x, y }),
                        Some(Size {
                            width: w,
                            height: h,
                        }),
                    )
                } else {
                    (None, None)
                }
            }
            Err(_) => (None, None),
        }
    }
}

fn get_children(element: &IUIAutomationElement) -> Vec<IUIAutomationElement> {
    unsafe {
        let mut children = Vec::new();
        let wrapper = TRUE_CONDITION.get_or_init(|| {
            let automation: IUIAutomation = windows::Win32::System::Com::CoCreateInstance(
                &CUIAutomation,
                None,
                windows::Win32::System::Com::CLSCTX_INPROC_SERVER,
            )
            .expect("COM already initialized");
            SendSyncCondition(
                automation
                    .CreateTrueCondition()
                    .expect("CreateTrueCondition"),
            )
        });
        if let Ok(array) = element.FindAll(TreeScope_Children, &wrapper.0) {
            if let Ok(len) = array.Length() {
                for i in 0..len {
                    if let Ok(child) = array.GetElement(i) {
                        children.push(child);
                    }
                }
            }
        }
        children
    }
}

pub fn find_all(root: &IUIAutomationElement, selector: &Selector) -> Vec<AccessibilityNode> {
    let mut results = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((root.clone(), 0u32));

    while let Some((element, depth)) = queue.pop_front() {
        if matches_selector(&element, selector) {
            results.push(element_to_node(&element, Some(0), 0));
        }

        if selector.max_depth.is_none_or(|max| depth < max) {
            for child in get_children(&element) {
                queue.push_back((child, depth + 1));
            }
        }
    }

    results
}

pub fn find_first(
    root: &IUIAutomationElement,
    selector: &Selector,
) -> Option<IUIAutomationElement> {
    let mut queue = VecDeque::new();
    queue.push_back((root.clone(), 0u32));

    while let Some((element, depth)) = queue.pop_front() {
        if matches_selector(&element, selector) {
            return Some(element);
        }

        if selector.max_depth.is_none_or(|max| depth < max) {
            for child in get_children(&element) {
                queue.push_back((child, depth + 1));
            }
        }
    }

    None
}

pub fn matches_selector(element: &IUIAutomationElement, selector: &Selector) -> bool {
    unsafe {
        if let Some(ref role) = selector.role {
            let control_type = element.CurrentControlType().unwrap_or_default();
            if map_control_type(control_type) != *role {
                return false;
            }
        }

        if let Some(ref name) = selector.name {
            let el_name = element.CurrentName().ok().map(|s| s.to_string());
            if el_name.as_deref() != Some(name.as_str()) {
                return false;
            }
        }

        if let Some(ref id) = selector.id {
            let el_id = element.CurrentAutomationId().ok().map(|s| s.to_string());
            if el_id.as_deref() != Some(id.as_str()) {
                return false;
            }
        }

        if let Some(ref sub) = selector.name_contains {
            let el_name = element
                .CurrentName()
                .ok()
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !el_name.to_lowercase().contains(&sub.to_lowercase()) {
                return false;
            }
        }

        if let Some(ref sub) = selector.id_contains {
            let el_id = element
                .CurrentAutomationId()
                .ok()
                .map(|s| s.to_string())
                .unwrap_or_default();
            if !el_id.to_lowercase().contains(&sub.to_lowercase()) {
                return false;
            }
        }

        true
    }
}

pub fn invoke_element(element: &IUIAutomationElement) -> bool {
    unsafe {
        // Try Invoke pattern (buttons, menu items, etc.)
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)
        {
            return pattern.Invoke().is_ok();
        }
        // Try SelectionItem pattern (tabs, list items, radio buttons)
        if let Ok(pattern) = element
            .GetCurrentPatternAs::<IUIAutomationSelectionItemPattern>(UIA_SelectionItemPatternId)
        {
            return pattern.Select().is_ok();
        }
        // Try Toggle pattern (checkboxes, toggle buttons)
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId)
        {
            return pattern.Toggle().is_ok();
        }
        false
    }
}

pub fn set_element_value(element: &IUIAutomationElement, value: &str) -> bool {
    unsafe {
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            return pattern.SetValue(&BSTR::from(value)).is_ok();
        }
        false
    }
}

pub fn scroll_element_into_view(element: &IUIAutomationElement) -> bool {
    unsafe {
        if let Ok(pattern) =
            element.GetCurrentPatternAs::<IUIAutomationScrollItemPattern>(UIA_ScrollItemPatternId)
        {
            return pattern.ScrollIntoView().is_ok();
        }
        false
    }
}

pub fn collect_text(element: &IUIAutomationElement, max_depth: Option<u32>, depth: u32) -> String {
    let mut parts = Vec::new();
    collect_text_recursive(element, max_depth, depth, &mut parts);
    parts.join("\n")
}

fn collect_text_recursive(
    element: &IUIAutomationElement,
    max_depth: Option<u32>,
    depth: u32,
    parts: &mut Vec<String>,
) {
    unsafe {
        let control_type = element.CurrentControlType().unwrap_or_default();
        if control_type == UIA_TextControlTypeId {
            if let Ok(name) = element.CurrentName() {
                let s = name.to_string();
                if !s.is_empty() {
                    parts.push(s);
                }
            }
        }

        if let Some(val) = get_value(element) {
            if !val.is_empty() && control_type != UIA_TextControlTypeId {
                parts.push(val);
            }
        }

        if max_depth.is_none_or(|max| depth < max) {
            for child in get_children(element) {
                collect_text_recursive(&child, max_depth, depth + 1, parts);
            }
        }
    }
}

// The UIA_*ControlTypeId values are PascalCase constants in windows-rs;
// clippy mistakes them for new bindings inside a `match` and emits
// `non_upper_case_globals` for each arm. Suppress at the function level.
#[allow(non_upper_case_globals)]
fn map_control_type(ct: UIA_CONTROLTYPE_ID) -> Role {
    match ct {
        UIA_ButtonControlTypeId => Role::Button,
        UIA_CalendarControlTypeId => Role::DatePicker,
        UIA_CheckBoxControlTypeId => Role::CheckBox,
        UIA_ComboBoxControlTypeId => Role::ComboBox,
        UIA_EditControlTypeId => Role::TextField,
        UIA_HyperlinkControlTypeId => Role::Link,
        UIA_ImageControlTypeId => Role::Image,
        UIA_ListItemControlTypeId => Role::ListItem,
        UIA_ListControlTypeId => Role::List,
        UIA_MenuControlTypeId => Role::Menu,
        UIA_MenuBarControlTypeId => Role::MenuBar,
        UIA_MenuItemControlTypeId => Role::MenuItem,
        UIA_ProgressBarControlTypeId => Role::ProgressIndicator,
        UIA_RadioButtonControlTypeId => Role::RadioButton,
        UIA_ScrollBarControlTypeId => Role::ScrollArea,
        UIA_SliderControlTypeId => Role::Slider,
        UIA_SpinnerControlTypeId => Role::Stepper,
        UIA_StatusBarControlTypeId => Role::Group,
        UIA_TabControlTypeId => Role::TabGroup,
        UIA_TabItemControlTypeId => Role::Tab,
        UIA_TextControlTypeId => Role::StaticText,
        UIA_ToolBarControlTypeId => Role::Toolbar,
        UIA_ToolTipControlTypeId => Role::Popover,
        UIA_TreeControlTypeId => Role::Tree,
        UIA_TreeItemControlTypeId => Role::TreeItem,
        UIA_GroupControlTypeId => Role::Group,
        UIA_ThumbControlTypeId => Role::Slider,
        UIA_DataGridControlTypeId => Role::Table,
        UIA_DataItemControlTypeId => Role::TableRow,
        UIA_DocumentControlTypeId => Role::TextArea,
        UIA_SplitButtonControlTypeId => Role::MenuButton,
        UIA_WindowControlTypeId => Role::Window,
        UIA_PaneControlTypeId => Role::Group,
        UIA_HeaderControlTypeId => Role::Group,
        UIA_HeaderItemControlTypeId => Role::TableColumn,
        UIA_TableControlTypeId => Role::Table,
        UIA_TitleBarControlTypeId => Role::Toolbar,
        UIA_SeparatorControlTypeId => Role::Group,
        _ => Role::Unknown,
    }
}
