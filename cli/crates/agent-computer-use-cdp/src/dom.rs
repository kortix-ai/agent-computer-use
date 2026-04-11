use crate::connection::CdpConnection;
use agent_computer_use_core::node::{AccessibilityNode, Point, Role, Size};
use agent_computer_use_core::Error;

pub const CDP_ID_PREFIX: &str = "__cdp:";

const DOM_WALKER_JS: &str = r#"
(function() {
    let counter = 0;

    const ARIA_ROLE_MAP = {
        'button': 'button', 'link': 'link', 'textbox': 'textfield',
        'checkbox': 'checkbox', 'radio': 'radiobutton', 'tab': 'tab',
        'tabpanel': 'tabpanel', 'tablist': 'tabgroup', 'menuitem': 'menuitem',
        'menu': 'menu', 'menubar': 'menubar', 'dialog': 'dialog',
        'alertdialog': 'dialog', 'slider': 'slider', 'switch': 'switch',
        'combobox': 'combobox', 'listbox': 'combobox', 'list': 'list',
        'listitem': 'listitem', 'option': 'listitem', 'heading': 'heading',
        'img': 'image', 'form': 'form', 'navigation': 'navigationbar',
        'search': 'search', 'banner': 'banner', 'main': 'main',
        'article': 'article', 'table': 'table', 'row': 'tablerow',
        'cell': 'group', 'grid': 'table', 'gridcell': 'group',
        'progressbar': 'progressindicator', 'tree': 'tree',
        'treeitem': 'treeitem', 'group': 'group', 'region': 'group',
        'separator': 'group', 'toolbar': 'toolbar', 'status': 'group',
        'scrollbar': 'slider', 'spinbutton': 'stepper',
        'complementary': 'group', 'contentinfo': 'group',
    };

    const TAG_ROLE_MAP = {
        'button': 'button', 'a': 'link', 'input': 'textfield',
        'textarea': 'textarea', 'select': 'combobox', 'img': 'image',
        'h1': 'heading', 'h2': 'heading', 'h3': 'heading',
        'h4': 'heading', 'h5': 'heading', 'h6': 'heading',
        'form': 'form', 'table': 'table', 'tr': 'tablerow',
        'td': 'group', 'th': 'group', 'li': 'listitem',
        'ul': 'list', 'ol': 'list', 'nav': 'navigationbar',
        'dialog': 'dialog', 'details': 'group', 'summary': 'button',
        'label': 'statictext', 'p': 'paragraph', 'header': 'banner',
        'footer': 'contentinfo', 'main': 'main', 'aside': 'group',
        'section': 'group', 'article': 'article', 'figure': 'group',
        'video': 'group', 'audio': 'group', 'canvas': 'image',
        'svg': 'image', 'iframe': 'webarea',
    };

    function roleFromEl(el) {
        const ariaRole = el.getAttribute('role');
        if (ariaRole && ARIA_ROLE_MAP[ariaRole]) return ARIA_ROLE_MAP[ariaRole];

        const tag = el.tagName.toLowerCase();
        if (tag === 'input') {
            const type = (el.type || 'text').toLowerCase();
            if (type === 'checkbox') return 'checkbox';
            if (type === 'radio') return 'radiobutton';
            if (type === 'range') return 'slider';
            if (type === 'password') return 'securetextfield';
            if (type === 'submit' || type === 'button' || type === 'reset') return 'button';
            if (type === 'search') return 'textfield';
            return 'textfield';
        }

        if (TAG_ROLE_MAP[tag]) return TAG_ROLE_MAP[tag];
        if (ariaRole) return ariaRole;
        return 'group';
    }

    function getName(el) {
        return el.getAttribute('aria-label')
            || el.getAttribute('aria-labelledby') && document.getElementById(el.getAttribute('aria-labelledby'))?.textContent?.trim()
            || el.getAttribute('alt')
            || el.getAttribute('title')
            || el.getAttribute('placeholder')
            || (el.labels && el.labels[0]?.textContent?.trim())
            || null;
    }

    function getTextContent(el) {
        let text = '';
        for (const child of el.childNodes) {
            if (child.nodeType === 3) text += child.textContent;
        }
        return text.trim().substring(0, 200) || null;
    }

    function getValue(el) {
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
            if (el.value !== undefined && el.value !== '') return String(el.value);
        }
        const checked = el.getAttribute('aria-checked');
        if (checked) return checked;
        const valuenow = el.getAttribute('aria-valuenow');
        if (valuenow) return valuenow;
        const selected = el.getAttribute('aria-selected');
        if (selected) return selected;
        const expanded = el.getAttribute('aria-expanded');
        if (expanded) return expanded;
        return null;
    }

    function isInteractive(el) {
        const tag = el.tagName.toLowerCase();
        if (['button', 'a', 'input', 'textarea', 'select', 'summary'].includes(tag)) return true;
        const role = el.getAttribute('role');
        if (role && ['button', 'link', 'textbox', 'checkbox', 'radio', 'tab',
            'menuitem', 'option', 'switch', 'slider', 'combobox', 'listitem',
            'treeitem', 'spinbutton', 'searchbox'].includes(role)) return true;
        if (el.getAttribute('tabindex') !== null) return true;
        if (el.getAttribute('contenteditable') === 'true') return true;
        if (el.onclick || el.getAttribute('data-testid') || el.getAttribute('data-qa')) return true;
        return false;
    }

    function isVisible(el) {
        if (el.offsetParent === null && el.tagName !== 'BODY' && el.tagName !== 'HTML') {
            const style = window.getComputedStyle(el);
            if (style.display === 'none') return false;
            if (style.position !== 'fixed' && style.position !== 'sticky') return false;
        }
        const style = window.getComputedStyle(el);
        if (style.visibility === 'hidden' || style.opacity === '0') return false;
        return true;
    }

    function walkNode(el, depth, maxDepth) {
        if (depth > maxDepth) return null;
        if (!el || el.nodeType !== 1) return null;
        if (!isVisible(el)) return null;

        const rect = el.getBoundingClientRect();
        const name = getName(el);
        const value = getValue(el);
        const origId = el.id || el.getAttribute('data-testid') || el.getAttribute('data-qa') || null;
        const directText = getTextContent(el);
        const interactive = isInteractive(el);

        const children = [];
        for (const child of el.children) {
            const node = walkNode(child, depth + 1, maxDepth);
            if (node) children.push(node);
        }

        const hasContent = name || origId || value || directText || interactive;

        if (!hasContent && children.length === 0) return null;
        if (!hasContent && children.length === 1) return children[0];

        let acId = null;
        if (interactive) {
            acId = String(counter++);
            el.setAttribute('data-acu', acId);
        }

        return {
            role: roleFromEl(el),
            name: name || directText || null,
            value: value,
            id: origId,
            acId: acId,
            x: rect.x + window.scrollX,
            y: rect.y + window.scrollY,
            w: rect.width,
            h: rect.height,
            focused: document.activeElement === el,
            enabled: !el.disabled,
            children: children,
        };
    }

    return walkNode(document.body, 0, 15);
})()
"#;

pub async fn get_dom_tree(
    conn: &CdpConnection,
    window_offset: (f64, f64),
) -> agent_computer_use_core::Result<AccessibilityNode> {
    let value = conn.evaluate(DOM_WALKER_JS).await?;

    if value.is_null() {
        return Ok(AccessibilityNode {
            role: Role::WebArea,
            name: Some("(empty)".into()),
            children: vec![],
            ..default_node()
        });
    }

    json_to_node(&value, window_offset).ok_or_else(|| Error::PlatformError {
        message: "failed to parse DOM tree from CDP".into(),
    })
}

fn json_to_node(v: &serde_json::Value, offset: (f64, f64)) -> Option<AccessibilityNode> {
    let role_str = v.get("role")?.as_str()?;
    let (ox, oy) = offset;

    let children: Vec<AccessibilityNode> = v
        .get("children")
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter().filter_map(|c| json_to_node(c, offset)).collect())
        .unwrap_or_default();

    let x = v.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = v.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let w = v.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let h = v.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let orig_id = v.get("id").and_then(|v| v.as_str()).map(String::from);
    let ac_id = v.get("acId").and_then(|v| v.as_str()).map(String::from);

    let id = match (&ac_id, &orig_id) {
        (Some(kid), Some(oid)) => Some(format!("{CDP_ID_PREFIX}{kid}:{oid}")),
        (Some(kid), None) => Some(format!("{CDP_ID_PREFIX}{kid}")),
        (None, Some(oid)) => Some(oid.clone()),
        (None, None) => None,
    };

    let has_position = w > 0.0 && h > 0.0;

    Some(AccessibilityNode {
        role: Role::parse(role_str),
        name: v.get("name").and_then(|v| v.as_str()).map(String::from),
        value: v.get("value").and_then(|v| v.as_str()).map(String::from),
        description: None,
        id,
        position: if has_position {
            Some(Point {
                x: x + ox,
                y: y + oy,
            })
        } else {
            None
        },
        size: if has_position {
            Some(Size {
                width: w,
                height: h,
            })
        } else {
            None
        },
        focused: v.get("focused").and_then(|v| v.as_bool()),
        enabled: v.get("enabled").and_then(|v| v.as_bool()),
        pid: None,
        children,
    })
}

pub fn extract_cdp_tag(id: &str) -> Option<&str> {
    let rest = id.strip_prefix(CDP_ID_PREFIX)?;
    Some(rest.split(':').next().unwrap_or(rest))
}

pub async fn click_by_tag(
    conn: &CdpConnection,
    tag: &str,
) -> agent_computer_use_core::Result<bool> {
    let js = format!(
        r#"(function() {{
            const el = document.querySelector('[data-acu="{tag}"]');
            if (!el) return false;
            el.scrollIntoView({{block: 'nearest', behavior: 'instant'}});
            el.focus();
            el.click();
            return true;
        }})()"#
    );
    let value = conn.evaluate(&js).await?;
    Ok(value.as_bool().unwrap_or(false))
}

pub async fn type_into_tag(
    conn: &CdpConnection,
    tag: &str,
    text: &str,
) -> agent_computer_use_core::Result<bool> {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n");
    let js = format!(
        r#"(function() {{
            const el = document.querySelector('[data-acu="{tag}"]');
            if (!el) return false;
            el.scrollIntoView({{block: 'nearest', behavior: 'instant'}});
            el.focus();
            if ('value' in el) {{
                const nativeSetter = Object.getOwnPropertyDescriptor(
                    window.HTMLInputElement.prototype, 'value'
                )?.set || Object.getOwnPropertyDescriptor(
                    window.HTMLTextAreaElement.prototype, 'value'
                )?.set;
                if (nativeSetter) {{
                    nativeSetter.call(el, '{escaped}');
                }} else {{
                    el.value = '{escaped}';
                }}
            }} else {{
                el.textContent = '{escaped}';
            }}
            el.dispatchEvent(new Event('input', {{bubbles: true}}));
            el.dispatchEvent(new Event('change', {{bubbles: true}}));
            return true;
        }})()"#
    );
    let value = conn.evaluate(&js).await?;
    Ok(value.as_bool().unwrap_or(false))
}

pub async fn get_value_by_tag(
    conn: &CdpConnection,
    tag: &str,
    window_offset: (f64, f64),
) -> agent_computer_use_core::Result<Option<AccessibilityNode>> {
    let js = format!(
        r#"(function() {{
            const el = document.querySelector('[data-acu="{tag}"]');
            if (!el) return null;
            const rect = el.getBoundingClientRect();
            const tag = el.tagName.toLowerCase();
            return {{
                role: el.getAttribute('role') || tag,
                name: el.getAttribute('aria-label') || el.innerText?.substring(0, 200) || null,
                value: el.value || el.getAttribute('aria-checked') || null,
                id: el.id || null,
                x: rect.x + window.scrollX,
                y: rect.y + window.scrollY,
                w: rect.width,
                h: rect.height,
                focused: document.activeElement === el,
                enabled: !el.disabled,
                children: [],
            }};
        }})()"#
    );
    let value = conn.evaluate(&js).await?;
    if value.is_null() {
        return Ok(None);
    }
    Ok(json_to_node(&value, window_offset))
}

pub async fn click_by_css(
    conn: &CdpConnection,
    css: &str,
) -> agent_computer_use_core::Result<bool> {
    let escaped = css.replace('\\', "\\\\").replace('\'', "\\'");
    let js = format!(
        r#"(function() {{
            const el = document.querySelector('{escaped}');
            if (!el) return false;
            el.scrollIntoView({{block: 'nearest', behavior: 'instant'}});
            el.focus();
            el.click();
            return true;
        }})()"#
    );
    let value = conn.evaluate(&js).await?;
    Ok(value.as_bool().unwrap_or(false))
}

pub async fn type_into_css(
    conn: &CdpConnection,
    css: &str,
    text: &str,
) -> agent_computer_use_core::Result<bool> {
    let escaped_sel = css.replace('\\', "\\\\").replace('\'', "\\'");
    let escaped_text = text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n");
    let js = format!(
        r#"(function() {{
            const el = document.querySelector('{escaped_sel}');
            if (!el) return false;
            el.focus();
            el.value = '{escaped_text}';
            el.dispatchEvent(new Event('input', {{bubbles: true}}));
            el.dispatchEvent(new Event('change', {{bubbles: true}}));
            return true;
        }})()"#
    );
    let value = conn.evaluate(&js).await?;
    Ok(value.as_bool().unwrap_or(false))
}

pub async fn query_selector_all(
    conn: &CdpConnection,
    css: &str,
    window_offset: (f64, f64),
) -> agent_computer_use_core::Result<Vec<AccessibilityNode>> {
    let escaped = css.replace('\\', "\\\\").replace('\'', "\\'");
    let js = format!(
        r#"(function() {{
            const els = document.querySelectorAll('{escaped}');
            return Array.from(els).map(el => {{
                const rect = el.getBoundingClientRect();
                const tag = el.tagName.toLowerCase();
                return {{
                    role: el.getAttribute('role') || tag,
                    name: el.getAttribute('aria-label') || el.innerText?.substring(0, 200) || null,
                    value: el.value || null,
                    id: el.id || el.getAttribute('data-testid') || null,
                    x: rect.x + window.scrollX,
                    y: rect.y + window.scrollY,
                    w: rect.width,
                    h: rect.height,
                    focused: document.activeElement === el,
                    enabled: !el.disabled,
                    children: [],
                }};
            }});
        }})()"#
    );

    let value = conn.evaluate(&js).await?;
    let arr = value.as_array().ok_or_else(|| Error::PlatformError {
        message: "CSS query did not return an array".into(),
    })?;

    Ok(arr
        .iter()
        .filter_map(|v| json_to_node(v, window_offset))
        .collect())
}

pub async fn dispatch_key(
    conn: &CdpConnection,
    key_expr: &str,
) -> agent_computer_use_core::Result<()> {
    let parts: Vec<&str> = key_expr.split('+').collect();
    let (key_name, modifiers) = parts.split_last().unwrap();
    let key_name = key_name.trim();

    let mut mod_flags = 0;
    for m in modifiers {
        match m.trim().to_lowercase().as_str() {
            "cmd" | "meta" | "command" => mod_flags |= 4,
            "ctrl" | "control" => mod_flags |= 2,
            "alt" | "option" => mod_flags |= 1,
            "shift" => mod_flags |= 8,
            _ => {}
        }
    }

    let key_lower = key_name.to_lowercase();
    let (key, code, key_code) = match key_lower.as_str() {
        "return" | "enter" => ("Enter", "Enter", 13),
        "escape" | "esc" => ("Escape", "Escape", 27),
        "tab" => ("Tab", "Tab", 9),
        "backspace" | "delete" => ("Backspace", "Backspace", 8),
        "space" => (" ", "Space", 32),
        "arrowup" | "up" => ("ArrowUp", "ArrowUp", 38),
        "arrowdown" | "down" => ("ArrowDown", "ArrowDown", 40),
        "arrowleft" | "left" => ("ArrowLeft", "ArrowLeft", 37),
        "arrowright" | "right" => ("ArrowRight", "ArrowRight", 39),
        "home" => ("Home", "Home", 36),
        "end" => ("End", "End", 35),
        "pageup" => ("PageUp", "PageUp", 33),
        "pagedown" => ("PageDown", "PageDown", 34),
        "f1" => ("F1", "F1", 112),
        "f2" => ("F2", "F2", 113),
        "f3" => ("F3", "F3", 114),
        "f4" => ("F4", "F4", 115),
        "f5" => ("F5", "F5", 116),
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            let code_num = c.to_ascii_uppercase() as i32;
            return dispatch_char_key(conn, c, code_num, mod_flags).await;
        }
        other => (other, "", 0),
    };

    conn.send(
        "Input.dispatchKeyEvent",
        Some(serde_json::json!({
            "type": "keyDown",
            "modifiers": mod_flags,
            "key": key,
            "code": code,
            "windowsVirtualKeyCode": key_code,
        })),
    )
    .await?;

    conn.send(
        "Input.dispatchKeyEvent",
        Some(serde_json::json!({
            "type": "keyUp",
            "modifiers": mod_flags,
            "key": key,
            "code": code,
            "windowsVirtualKeyCode": key_code,
        })),
    )
    .await?;

    Ok(())
}

async fn dispatch_char_key(
    conn: &CdpConnection,
    c: char,
    code_num: i32,
    mod_flags: i32,
) -> agent_computer_use_core::Result<()> {
    let key = c.to_string();
    let code = format!("Key{}", c.to_ascii_uppercase());

    conn.send(
        "Input.dispatchKeyEvent",
        Some(serde_json::json!({
            "type": "keyDown",
            "modifiers": mod_flags,
            "key": key,
            "code": code,
            "windowsVirtualKeyCode": code_num,
        })),
    )
    .await?;

    conn.send(
        "Input.dispatchKeyEvent",
        Some(serde_json::json!({
            "type": "keyUp",
            "modifiers": mod_flags,
            "key": key,
            "code": code,
            "windowsVirtualKeyCode": code_num,
        })),
    )
    .await?;

    Ok(())
}

pub async fn insert_text(conn: &CdpConnection, text: &str) -> agent_computer_use_core::Result<()> {
    conn.send(
        "Input.insertText",
        Some(serde_json::json!({ "text": text })),
    )
    .await?;
    Ok(())
}

pub async fn scroll_page(
    conn: &CdpConnection,
    direction: &str,
    amount: u32,
    at_selector: Option<&str>,
) -> agent_computer_use_core::Result<()> {
    let pixels = (amount as i32) * 120;
    let (dx, dy) = match direction {
        "up" => (0, -pixels),
        "down" => (0, pixels),
        "left" => (-pixels, 0),
        "right" => (pixels, 0),
        _ => (0, 0),
    };

    let js = if let Some(sel) = at_selector {
        let escaped = sel.replace('\\', "\\\\").replace('\'', "\\'");
        format!(
            r#"(function() {{
                let el = document.querySelector('{escaped}');
                if (!el) return false;
                while (el && el !== document.body) {{
                    if (el.scrollHeight > el.clientHeight || el.scrollWidth > el.clientWidth) {{
                        const s = window.getComputedStyle(el);
                        if (s.overflow !== 'visible' && s.overflow !== 'hidden') {{
                            el.scrollBy({{ left: {dx}, top: {dy}, behavior: 'instant' }});
                            return true;
                        }}
                    }}
                    el = el.parentElement;
                }}
                window.scrollBy({{ left: {dx}, top: {dy}, behavior: 'instant' }});
                return true;
            }})()"#
        )
    } else {
        format!(
            r#"(function() {{
                const all = document.querySelectorAll('*');
                let best = null;
                let bestScore = 0;
                for (const el of all) {{
                    if (el.scrollHeight > el.clientHeight + 20) {{
                        const rect = el.getBoundingClientRect();
                        const visible = rect.width > 100 && rect.height > 100;
                        if (!visible) continue;
                        const score = (el.scrollHeight - el.clientHeight) * rect.width;
                        if (score > bestScore) {{
                            bestScore = score;
                            best = el;
                        }}
                    }}
                }}
                if (best) {{
                    best.scrollBy({{ left: {dx}, top: {dy}, behavior: 'instant' }});
                    return true;
                }}
                window.scrollBy({{ left: {dx}, top: {dy}, behavior: 'instant' }});
                return true;
            }})()"#
        )
    };

    conn.evaluate(&js).await?;
    Ok(())
}

pub async fn get_page_text(conn: &CdpConnection) -> agent_computer_use_core::Result<String> {
    let value = conn.evaluate("document.body.innerText").await?;
    Ok(value.as_str().unwrap_or("").to_string())
}

fn default_node() -> AccessibilityNode {
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
