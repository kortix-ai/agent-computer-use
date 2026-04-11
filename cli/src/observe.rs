use agent_computer_use_core::node::{AccessibilityNode, Role};
use agent_computer_use_core::Platform;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

struct FlatNode {
    path: Vec<usize>,
    depth: usize,
    has_children: bool,
    is_expanded: bool,
    role: Role,
    name: Option<String>,
    value: Option<String>,
    id: Option<String>,
    position: Option<(f64, f64)>,
    size: Option<(f64, f64)>,
    #[allow(dead_code)]
    child_count: usize,
}

struct AppState {
    tree: Option<AccessibilityNode>,
    flat_nodes: Vec<FlatNode>,
    list_state: ListState,
    expanded: HashSet<Vec<usize>>,
    search_query: String,
    search_mode: bool,
    last_refresh: Instant,
    status_msg: String,
    node_count: usize,
}

impl AppState {
    fn new() -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(vec![]);
        Self {
            tree: None,
            flat_nodes: Vec::new(),
            list_state: ListState::default(),
            expanded,
            search_query: String::new(),
            search_mode: false,
            last_refresh: Instant::now(),
            status_msg: String::new(),
            node_count: 0,
        }
    }

    async fn refresh(&mut self, platform: &dyn Platform, app: Option<&str>, depth: u32) {
        match platform.tree(app, Some(depth)).await {
            Ok(tree) => {
                self.node_count = tree.node_count();
                self.tree = Some(tree);
                self.rebuild_flat();
                self.status_msg = format!("{} nodes", self.node_count);
            }
            Err(e) => {
                self.status_msg = format!("error: {e}");
            }
        }
        self.last_refresh = Instant::now();
    }

    fn rebuild_flat(&mut self) {
        self.flat_nodes.clear();
        if let Some(ref tree) = self.tree {
            flatten_recursive(
                tree,
                &[],
                0,
                &self.expanded,
                &self.search_query,
                &mut self.flat_nodes,
            );
        }
        if let Some(sel) = self.list_state.selected() {
            if sel >= self.flat_nodes.len() && !self.flat_nodes.is_empty() {
                self.list_state.select(Some(self.flat_nodes.len() - 1));
            }
        }
    }

    fn toggle_expand(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(node) = self.flat_nodes.get(idx) {
                if node.has_children {
                    let path = node.path.clone();
                    if self.expanded.contains(&path) {
                        self.expanded.remove(&path);
                    } else {
                        self.expanded.insert(path);
                    }
                    self.rebuild_flat();
                }
            }
        }
    }

    fn collapse(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(node) = self.flat_nodes.get(idx) {
                let path = node.path.clone();
                if self.expanded.contains(&path) {
                    self.expanded.remove(&path);
                    self.rebuild_flat();
                } else if path.len() > 1 {
                    let parent = path[..path.len() - 1].to_vec();
                    if let Some(parent_idx) = self.flat_nodes.iter().position(|n| n.path == parent)
                    {
                        self.list_state.select(Some(parent_idx));
                    }
                }
            }
        }
    }

    fn copy_selector(&self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(node) = self.flat_nodes.get(idx) {
                #[cfg(target_os = "macos")]
                {
                    let dsl = build_selector_dsl(node);
                    let _ = std::process::Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .and_then(|mut child| {
                            use std::io::Write;
                            if let Some(ref mut stdin) = child.stdin {
                                stdin.write_all(dsl.as_bytes())?;
                            }
                            child.wait()
                        });
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = node;
                }
            }
        }
    }

    fn selected_selector_dsl(&self) -> String {
        if let Some(idx) = self.list_state.selected() {
            if let Some(node) = self.flat_nodes.get(idx) {
                return build_selector_dsl(node);
            }
        }
        String::new()
    }
}

fn flatten_recursive(
    node: &AccessibilityNode,
    path: &[usize],
    depth: usize,
    expanded: &HashSet<Vec<usize>>,
    search: &str,
    result: &mut Vec<FlatNode>,
) {
    let node_path = path.to_vec();
    let is_expanded = expanded.contains(&node_path);

    let matches_search = search.is_empty() || node_matches_search(node, search);

    if !search.is_empty() && !matches_search && !subtree_has_match(node, search) {
        return;
    }

    result.push(FlatNode {
        path: node_path.clone(),
        depth,
        has_children: !node.children.is_empty(),
        is_expanded,
        role: node.role.clone(),
        name: node.name.clone(),
        value: node.value.clone(),
        id: node.id.clone(),
        position: node.position.map(|p| (p.x, p.y)),
        size: node.size.map(|s| (s.width, s.height)),
        child_count: node.children.len(),
    });

    if is_expanded || (!search.is_empty() && subtree_has_match(node, search)) {
        for (i, child) in node.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            flatten_recursive(child, &child_path, depth + 1, expanded, search, result);
        }
    }
}

fn node_matches_search(node: &AccessibilityNode, query: &str) -> bool {
    let q = query.to_lowercase();
    let role_str = format!("{:?}", node.role).to_lowercase();
    if role_str.contains(&q) {
        return true;
    }
    if let Some(ref name) = node.name {
        if name.to_lowercase().contains(&q) {
            return true;
        }
    }
    if let Some(ref id) = node.id {
        if id.to_lowercase().contains(&q) {
            return true;
        }
    }
    if let Some(ref value) = node.value {
        if value.to_lowercase().contains(&q) {
            return true;
        }
    }
    false
}

fn subtree_has_match(node: &AccessibilityNode, query: &str) -> bool {
    if node_matches_search(node, query) {
        return true;
    }
    node.children.iter().any(|c| subtree_has_match(c, query))
}

fn build_selector_dsl(node: &FlatNode) -> String {
    let mut parts = Vec::new();

    if let Some(ref id) = node.id {
        if !id.is_empty() && !id.starts_with("_NS:") {
            parts.push(format!("id=\"{id}\""));
            return parts.join(" ");
        }
    }

    let role_str = format!("{:?}", node.role);
    let role_lower = role_str.to_lowercase();
    if !matches!(node.role, Role::Unknown | Role::Other(_)) {
        parts.push(format!("role={role_lower}"));
    }

    if let Some(ref name) = node.name {
        if !name.is_empty() {
            parts.push(format!("name=\"{name}\""));
        }
    }

    if parts.is_empty() {
        "?".into()
    } else {
        parts.join(" ")
    }
}

fn role_icon(role: &Role) -> &'static str {
    match role {
        Role::Application => "app",
        Role::Window => "win",
        Role::Button => "btn",
        Role::TextField | Role::TextArea | Role::SecureTextField => "txt",
        Role::CheckBox => "chk",
        Role::RadioButton => "rad",
        Role::Menu | Role::MenuBar => "mnu",
        Role::MenuItem | Role::MenuButton => "mni",
        Role::StaticText => "lbl",
        Role::Image | Role::Icon => "img",
        Role::Link => "lnk",
        Role::Tab | Role::TabGroup => "tab",
        Role::List | Role::ListItem => "lst",
        Role::Table | Role::TableRow | Role::TableColumn => "tbl",
        Role::Group | Role::SplitGroup => "grp",
        Role::Toolbar => "bar",
        Role::ScrollArea => "scr",
        Role::Slider => "sld",
        Role::Switch => "swt",
        Role::ComboBox | Role::PopUpButton => "cmb",
        Role::Dialog | Role::Sheet => "dlg",
        Role::ProgressIndicator | Role::BusyIndicator => "prg",
        Role::WebArea => "web",
        Role::Heading => "h",
        Role::Form => "frm",
        _ => "···",
    }
}

pub async fn run_observe(
    platform: &dyn Platform,
    app: Option<String>,
    depth: u32,
    refresh_interval: Duration,
) -> agent_computer_use_core::Result<()> {
    terminal::enable_raw_mode().map_err(|e| agent_computer_use_core::Error::PlatformError {
        message: format!("failed to enable raw mode: {e}"),
    })?;

    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).map_err(|e| {
        agent_computer_use_core::Error::PlatformError {
            message: format!("failed to enter alternate screen: {e}"),
        }
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| agent_computer_use_core::Error::PlatformError {
            message: format!("failed to create terminal: {e}"),
        })?;

    let mut state = AppState::new();
    state.refresh(platform, app.as_deref(), depth).await;
    if !state.flat_nodes.is_empty() {
        state.list_state.select(Some(0));
    }

    let result = run_event_loop(
        &mut terminal,
        &mut state,
        platform,
        app.as_deref(),
        depth,
        refresh_interval,
    )
    .await;

    let _ = terminal::disable_raw_mode();
    let _ = crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen);

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    platform: &dyn Platform,
    app: Option<&str>,
    depth: u32,
    refresh_interval: Duration,
) -> agent_computer_use_core::Result<()> {
    loop {
        terminal.draw(|f| render(f, state)).map_err(|e| {
            agent_computer_use_core::Error::PlatformError {
                message: format!("render error: {e}"),
            }
        })?;

        let time_until_refresh = refresh_interval
            .checked_sub(state.last_refresh.elapsed())
            .unwrap_or(Duration::ZERO);

        let has_event = event::poll(time_until_refresh).map_err(|e| {
            agent_computer_use_core::Error::PlatformError {
                message: format!("event poll error: {e}"),
            }
        })?;

        if has_event {
            let ev = event::read().map_err(|e| agent_computer_use_core::Error::PlatformError {
                message: format!("event read error: {e}"),
            })?;

            if let Event::Key(key) = ev {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if state.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            state.search_mode = false;
                            state.search_query.clear();
                            state.rebuild_flat();
                        }
                        KeyCode::Enter => {
                            state.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            state.search_query.pop();
                            state.rebuild_flat();
                        }
                        KeyCode::Char(c) => {
                            state.search_query.push(c);
                            state.rebuild_flat();
                            if !state.flat_nodes.is_empty() {
                                state.list_state.select(Some(0));
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('/') => {
                            state.search_mode = true;
                            state.search_query.clear();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = state.list_state.selected().unwrap_or(0);
                            if i > 0 {
                                state.list_state.select(Some(i - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = state.list_state.selected().unwrap_or(0);
                            if i + 1 < state.flat_nodes.len() {
                                state.list_state.select(Some(i + 1));
                            }
                        }
                        KeyCode::Enter | KeyCode::Right => {
                            state.toggle_expand();
                        }
                        KeyCode::Left => {
                            state.collapse();
                        }
                        KeyCode::Char('y') => {
                            state.copy_selector();
                            let dsl = state.selected_selector_dsl();
                            state.status_msg = format!("copied: {dsl}");
                        }
                        KeyCode::Char('r') => {
                            state.refresh(platform, app, depth).await;
                        }
                        _ => {}
                    }
                }
            }
        } else {
            state.refresh(platform, app, depth).await;
        }
    }
    Ok(())
}

fn render(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = format!(
        " agent-computer-use observe | {} | refreshed {:.0}s ago",
        state.status_msg,
        state.last_refresh.elapsed().as_secs_f64()
    );
    let title_bar =
        Paragraph::new(title).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(title_bar, chunks[0]);

    let items: Vec<ListItem> = state
        .flat_nodes
        .iter()
        .map(|node| {
            let indent = "  ".repeat(node.depth);
            let arrow = if node.has_children {
                if node.is_expanded {
                    "v "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            let icon = role_icon(&node.role);
            let name_part = node
                .name
                .as_ref()
                .map(|n| format!(" \"{n}\""))
                .unwrap_or_default();
            let value_part = node
                .value
                .as_ref()
                .map(|v| {
                    let truncated = if v.len() > 30 {
                        format!("{}…", &v[..30])
                    } else {
                        v.clone()
                    };
                    format!("  val=\"{truncated}\"")
                })
                .unwrap_or_default();
            let id_part = node
                .id
                .as_ref()
                .filter(|id| !id.is_empty() && !id.starts_with("_NS:"))
                .map(|id| format!("  id={id}"))
                .unwrap_or_default();
            let pos_part = node
                .position
                .map(|(x, y)| format!("  ({x:.0},{y:.0})"))
                .unwrap_or_default();
            let size_part = node
                .size
                .map(|(w, h)| format!(" {w:.0}x{h:.0}"))
                .unwrap_or_default();

            let line = format!(
                "{indent}{arrow}[{icon}]{name_part}{value_part}{id_part}{pos_part}{size_part}"
            );

            let style = if !state.search_query.is_empty()
                && line
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
            {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let tree_list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol(">> ");
    f.render_stateful_widget(tree_list, chunks[1], &mut state.list_state.clone());

    let bottom = if state.search_mode {
        format!(" /{}█", state.search_query)
    } else {
        let dsl = state.selected_selector_dsl();
        format!(
            " j/k:move  enter:expand  left:collapse  /:search  y:copy  r:refresh  q:quit  | {dsl}"
        )
    };
    let bottom_bar =
        Paragraph::new(bottom).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(bottom_bar, chunks[2]);
}
