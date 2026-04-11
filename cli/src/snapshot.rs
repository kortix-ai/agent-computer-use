use agent_computer_use_core::node::{AccessibilityNode, Role};
use agent_computer_use_core::selector::{Selector, SelectorChain};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const REFS_PATH: &str = ".agent-cu/refs.json";

#[derive(Debug, Serialize)]
pub struct SnapshotResult {
    pub snapshot: String,
    pub refs: HashMap<String, RefEntry>,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefEntry {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<usize>,
}

use agent_computer_use_core::element::is_interactive;

pub fn create_snapshot(
    tree: &AccessibilityNode,
    app_name: Option<&str>,
    interactive_only: bool,
    compact: bool,
) -> SnapshotResult {
    let mut ctx = WalkCtx {
        app_name,
        interactive_only,
        compact,
        counter: 1,
        refs: HashMap::new(),
        lines: Vec::new(),
        current_path: Vec::new(),
    };

    walk_node(tree, 0, &mut ctx);
    let node_count = tree.node_count();

    SnapshotResult {
        snapshot: ctx.lines.join("\n"),
        refs: ctx.refs,
        node_count,
    }
}

struct WalkCtx<'a> {
    app_name: Option<&'a str>,
    interactive_only: bool,
    compact: bool,
    counter: u32,
    refs: HashMap<String, RefEntry>,
    lines: Vec<String>,
    current_path: Vec<usize>,
}

fn walk_node(node: &AccessibilityNode, depth: usize, ctx: &mut WalkCtx<'_>) {
    let has_name = node.name.as_ref().is_some_and(|n| !n.is_empty());
    let has_id = node.id.as_ref().is_some_and(|i| !i.is_empty());
    let has_children = !node.children.is_empty();

    if ctx.compact && !has_name && !has_id && !has_children && !is_interactive(&node.role) {
        return;
    }

    let is_cdp = node.id.as_ref().is_some_and(|id| id.starts_with("__cdp:"));

    let assign_ref = if ctx.interactive_only {
        is_interactive(&node.role) || is_cdp
    } else {
        true
    };

    let indent = "  ".repeat(depth);
    let role_str = format!("{:?}", node.role).to_lowercase();

    let ref_label = if assign_ref {
        let ref_id = format!("e{}", ctx.counter);
        ctx.counter += 1;

        ctx.refs.insert(
            ref_id.clone(),
            RefEntry {
                role: role_str.clone(),
                name: node.name.clone(),
                id: node.id.clone(),
                app: ctx.app_name.map(|s| s.to_string()),
                path: ctx.current_path.clone(),
            },
        );

        format!("[@{}] ", ref_id)
    } else {
        String::new()
    };

    let name_part = node
        .name
        .as_ref()
        .filter(|n| !n.is_empty())
        .map(|n| format!(" \"{}\"", n))
        .unwrap_or_default();

    let value_part = node
        .value
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| {
            let truncated = if v.len() > 40 {
                let end = v
                    .char_indices()
                    .map(|(i, _)| i)
                    .take_while(|&i| i <= 40)
                    .last()
                    .unwrap_or(0);
                format!("{}…", &v[..end])
            } else {
                v.clone()
            };
            format!(" val=\"{}\"", truncated)
        })
        .unwrap_or_default();

    let id_part = node
        .id
        .as_ref()
        .filter(|i| !i.is_empty() && !i.starts_with("_NS:"))
        .map(|i| format!(" id={}", i))
        .unwrap_or_default();

    ctx.lines.push(format!(
        "{indent}{ref_label}{role_str}{name_part}{value_part}{id_part}"
    ));

    for (i, child) in node.children.iter().enumerate() {
        ctx.current_path.push(i);
        walk_node(child, depth + 1, ctx);
        ctx.current_path.pop();
    }
}

const STALE_THRESHOLD_SECS: u64 = 300;

#[derive(Debug, Serialize, Deserialize)]
struct RefsCache {
    #[serde(default)]
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    refs: HashMap<String, RefEntry>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn home_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
}

pub fn save_refs(
    refs: &HashMap<String, RefEntry>,
    app: Option<&str>,
) -> agent_computer_use_core::Result<()> {
    let home = home_dir();
    let dir = format!("{home}/{}", REFS_PATH.rsplit_once('/').unwrap().0);
    std::fs::create_dir_all(&dir).map_err(agent_computer_use_core::Error::Io)?;

    let cache = RefsCache {
        timestamp: now_secs(),
        app: app.map(|s| s.to_string()),
        refs: refs.clone(),
    };

    let path = format!("{home}/{REFS_PATH}");
    let json = serde_json::to_string_pretty(&cache)
        .map_err(agent_computer_use_core::Error::Serialization)?;
    std::fs::write(&path, json).map_err(agent_computer_use_core::Error::Io)?;

    tracing::debug!("saved {} refs to {}", refs.len(), path);
    Ok(())
}

fn load_refs() -> agent_computer_use_core::Result<HashMap<String, RefEntry>> {
    let home = home_dir();
    let path = format!("{home}/{REFS_PATH}");

    let contents = std::fs::read_to_string(&path).map_err(|_| {
        agent_computer_use_core::Error::PlatformError {
            message: "no snapshot refs cached — run `agent-computer-use snapshot` first".into(),
        }
    })?;

    if let Ok(cache) = serde_json::from_str::<RefsCache>(&contents) {
        let age = now_secs().saturating_sub(cache.timestamp);
        if age > STALE_THRESHOLD_SECS {
            tracing::warn!(
                "refs are {}s old (snapshot taken {}s ago) — consider re-running `agent-computer-use snapshot`",
                age,
                age
            );
            eprintln!(
                "warning: snapshot refs are {age}s old — run `agent-computer-use snapshot` to refresh"
            );
        }
        return Ok(cache.refs);
    }

    serde_json::from_str(&contents).map_err(|e| agent_computer_use_core::Error::PlatformError {
        message: format!("invalid refs cache: {e}"),
    })
}

pub fn resolve_ref(ref_str: &str) -> agent_computer_use_core::Result<SelectorChain> {
    let ref_id = ref_str.strip_prefix('@').unwrap_or(ref_str);
    let refs = load_refs()?;

    let entry =
        refs.get(ref_id)
            .ok_or_else(|| agent_computer_use_core::Error::ElementNotFound {
                message: format!(
                    "ref '@{ref_id}' not found — run `agent-computer-use snapshot` to refresh"
                ),
            })?;

    let mut selector = Selector::new();
    selector.app = entry.app.clone();
    selector.name = entry.name.clone();
    selector.id = entry.id.clone();

    if let Some(role) = parse_role_from_string(&entry.role) {
        selector.role = Some(role);
    }

    if !entry.path.is_empty() {
        selector.path = Some(entry.path.clone());
    }

    tracing::debug!("resolved @{} → {:?}", ref_id, selector);
    Ok(SelectorChain::single(selector))
}

fn parse_role_from_string(s: &str) -> Option<Role> {
    Some(Role::parse(s))
}
