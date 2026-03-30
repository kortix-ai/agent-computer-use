use agent_click_core::element::{collect_text, is_interactive, is_visible, rank};
use agent_click_core::node::{AccessibilityNode, Point, Role, Size};
use agent_click_core::selector::Selector;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ---------------------------------------------------------------------------
// Helpers to build mock nodes
// ---------------------------------------------------------------------------

fn make_node(role: Role, name: Option<&str>) -> AccessibilityNode {
    AccessibilityNode {
        role,
        name: name.map(|s| s.to_string()),
        value: None,
        description: None,
        id: None,
        position: Some(Point { x: 100.0, y: 200.0 }),
        size: Some(Size {
            width: 80.0,
            height: 30.0,
        }),
        focused: None,
        enabled: Some(true),
        pid: None,
        children: Vec::new(),
    }
}

fn make_tree(depth: usize, breadth: usize) -> AccessibilityNode {
    fn build(depth: usize, breadth: usize, idx: usize) -> AccessibilityNode {
        let role = match idx % 5 {
            0 => Role::Group,
            1 => Role::Button,
            2 => Role::StaticText,
            3 => Role::TextField,
            _ => Role::Link,
        };
        let children = if depth == 0 {
            Vec::new()
        } else {
            (0..breadth).map(|i| build(depth - 1, breadth, i)).collect()
        };
        AccessibilityNode {
            role,
            name: Some(format!("node-{}-{}", depth, idx)),
            value: if idx % 3 == 0 {
                Some("some value".to_string())
            } else {
                None
            },
            description: None,
            id: Some(format!("id-{}-{}", depth, idx)),
            position: Some(Point {
                x: (idx * 10) as f64,
                y: (depth * 20) as f64,
            }),
            size: Some(Size {
                width: 80.0,
                height: 30.0,
            }),
            focused: None,
            enabled: Some(true),
            pid: None,
            children,
        }
    }
    build(depth, breadth, 0)
}

// ---------------------------------------------------------------------------
// Role::parse benchmarks
// ---------------------------------------------------------------------------

fn bench_role_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("role_parse");

    group.bench_function("known_role_button", |b| {
        b.iter(|| Role::parse(black_box("button")));
    });

    group.bench_function("known_role_textfield", |b| {
        b.iter(|| Role::parse(black_box("textfield")));
    });

    group.bench_function("known_role_statictext", |b| {
        b.iter(|| Role::parse(black_box("statictext")));
    });

    group.bench_function("known_role_mixed_case", |b| {
        b.iter(|| Role::parse(black_box("Button")));
    });

    group.bench_function("unknown_role", |b| {
        b.iter(|| Role::parse(black_box("customwidget")));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Selector::matches benchmarks
// ---------------------------------------------------------------------------

fn bench_selector_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("selector_matches");

    let node = make_node(Role::Button, Some("Submit"));

    group.bench_function("match_role_only", |b| {
        let sel = Selector::new().with_role(Role::Button);
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.bench_function("match_role_and_name", |b| {
        let sel = Selector::new().with_role(Role::Button).with_name("Submit");
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.bench_function("match_name_contains", |b| {
        let sel = Selector::new().with_name_contains("ubmi");
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.bench_function("no_match_wrong_role", |b| {
        let sel = Selector::new().with_role(Role::TextField);
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.bench_function("no_match_wrong_name", |b| {
        let sel = Selector::new().with_role(Role::Button).with_name("Cancel");
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.bench_function("match_by_id", |b| {
        let mut node_with_id = make_node(Role::Button, Some("Submit"));
        node_with_id.id = Some("btn-submit".to_string());
        let sel = Selector::new().with_id("btn-submit");
        b.iter(|| sel.matches(black_box(&node_with_id)));
    });

    group.bench_function("empty_selector_matches_all", |b| {
        let sel = Selector::new();
        b.iter(|| sel.matches(black_box(&node)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Element utility benchmarks
// ---------------------------------------------------------------------------

fn bench_element_utils(c: &mut Criterion) {
    let mut group = c.benchmark_group("element_utils");

    group.bench_function("is_interactive_button", |b| {
        b.iter(|| is_interactive(black_box(&Role::Button)));
    });

    group.bench_function("is_interactive_group", |b| {
        b.iter(|| is_interactive(black_box(&Role::Group)));
    });

    let visible_node = make_node(Role::Button, Some("OK"));
    group.bench_function("is_visible_true", |b| {
        b.iter(|| is_visible(black_box(&visible_node)));
    });

    let mut invisible_node = make_node(Role::Button, Some("Hidden"));
    invisible_node.size = Some(Size {
        width: 0.0,
        height: 0.0,
    });
    group.bench_function("is_visible_false_zero_size", |b| {
        b.iter(|| is_visible(black_box(&invisible_node)));
    });

    let mut no_pos_node = make_node(Role::Button, Some("NoPos"));
    no_pos_node.position = None;
    group.bench_function("is_visible_false_no_position", |b| {
        b.iter(|| is_visible(black_box(&no_pos_node)));
    });

    group.bench_function("rank_interactive", |b| {
        let node = make_node(Role::Button, Some("Click"));
        b.iter(|| rank(black_box(&node)));
    });

    group.bench_function("rank_non_interactive", |b| {
        let node = make_node(Role::Group, Some("Container"));
        b.iter(|| rank(black_box(&node)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Node tree operation benchmarks
// ---------------------------------------------------------------------------

fn bench_node_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_ops");

    // Small tree: depth=3, breadth=3 => ~40 nodes
    let small_tree = make_tree(3, 3);
    // Medium tree: depth=4, breadth=4 => ~341 nodes
    let medium_tree = make_tree(4, 4);

    group.bench_function("node_count_small", |b| {
        b.iter(|| black_box(&small_tree).node_count());
    });

    group.bench_function("node_count_medium", |b| {
        b.iter(|| black_box(&medium_tree).node_count());
    });

    group.bench_function("walk_path_depth3", |b| {
        b.iter(|| small_tree.walk_path(black_box(&[1, 2, 0])));
    });

    group.bench_function("walk_path_depth4", |b| {
        b.iter(|| medium_tree.walk_path(black_box(&[1, 2, 0, 1])));
    });

    group.bench_function("find_all_buttons_small", |b| {
        b.iter(|| {
            small_tree.find_all(&|n: &AccessibilityNode| n.role == Role::Button);
        });
    });

    group.bench_function("find_all_buttons_medium", |b| {
        b.iter(|| {
            medium_tree.find_all(&|n: &AccessibilityNode| n.role == Role::Button);
        });
    });

    group.bench_function("find_first_textfield_small", |b| {
        b.iter(|| {
            small_tree.find_first(&|n: &AccessibilityNode| n.role == Role::TextField);
        });
    });

    group.bench_function("center_calculation", |b| {
        let node = make_node(Role::Button, Some("OK"));
        b.iter(|| black_box(&node).center());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// collect_text benchmarks
// ---------------------------------------------------------------------------

fn bench_collect_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("collect_text");

    // Build a tree with StaticText leaves
    let tree = AccessibilityNode {
        role: Role::Group,
        name: Some("container".to_string()),
        value: None,
        description: None,
        id: None,
        position: Some(Point { x: 0.0, y: 0.0 }),
        size: Some(Size {
            width: 400.0,
            height: 300.0,
        }),
        focused: None,
        enabled: Some(true),
        pid: None,
        children: (0..10)
            .map(|i| AccessibilityNode {
                role: Role::StaticText,
                name: Some(format!("Line {} of text content", i)),
                value: None,
                description: None,
                id: None,
                position: Some(Point {
                    x: 0.0,
                    y: (i * 20) as f64,
                }),
                size: Some(Size {
                    width: 400.0,
                    height: 18.0,
                }),
                focused: None,
                enabled: None,
                pid: None,
                children: Vec::new(),
            })
            .collect(),
    };

    group.bench_function("collect_10_lines", |b| {
        b.iter(|| collect_text(black_box(&tree)));
    });

    group.bench_function("collect_nested_tree", |b| {
        let nested = make_tree(3, 3);
        b.iter(|| collect_text(black_box(&nested)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_role_parse,
    bench_selector_matches,
    bench_element_utils,
    bench_node_ops,
    bench_collect_text,
);
criterion_main!(benches);
