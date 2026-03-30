use criterion::{black_box, criterion_group, criterion_main, Criterion};

// The selector_dsl module is private in the agent-click binary crate.
// We test the public interface by importing agent_click_core types and
// re-implementing the DSL parser inline for benchmarking purposes.
// Since the binary re-exports the parse function, we include the
// source directly.
#[path = "../src/selector_dsl.rs"]
mod selector_dsl;

#[path = "../src/snapshot.rs"]
mod snapshot;

use agent_click_core::node::{AccessibilityNode, Point, Role, Size};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
// Selector DSL parsing benchmarks
// ---------------------------------------------------------------------------

fn bench_selector_dsl(c: &mut Criterion) {
    let mut group = c.benchmark_group("selector_dsl");

    group.bench_function("simple_role", |b| {
        b.iter(|| selector_dsl::parse(black_box("button")));
    });

    group.bench_function("role_key_value", |b| {
        b.iter(|| selector_dsl::parse(black_box("role=button")));
    });

    group.bench_function("role_and_quoted_name", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"role=button name="Submit Form""#)));
    });

    group.bench_function("shorthand_role_and_name", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"button "Submit""#)));
    });

    group.bench_function("name_contains", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"name~="email""#)));
    });

    group.bench_function("app_scoped", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"app="Firefox" role=button name="Submit""#)));
    });

    group.bench_function("with_index", |b| {
        b.iter(|| selector_dsl::parse(black_box("role=button index=2")));
    });

    group.bench_function("with_depth", |b| {
        b.iter(|| selector_dsl::parse(black_box("role=button depth=5")));
    });

    group.bench_function("chain_two", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"role=form >> role=button name="Submit""#)));
    });

    group.bench_function("chain_three", |b| {
        b.iter(|| {
            selector_dsl::parse(black_box(
                r#"role=window >> role=group >> role=button name="OK""#,
            ))
        });
    });

    group.bench_function("complex_chain", |b| {
        b.iter(|| {
            selector_dsl::parse(black_box(
                r#"app="Safari" role=webarea >> role=form id="login" >> role=button name="Sign In" index=0"#,
            ))
        });
    });

    group.bench_function("id_selector", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"id="login-btn""#)));
    });

    group.bench_function("escaped_quotes", |b| {
        b.iter(|| selector_dsl::parse(black_box(r#"name="say \"hello\"""#)));
    });

    group.bench_function("unquoted_values", |b| {
        b.iter(|| selector_dsl::parse(black_box("role=button name=Submit")));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Snapshot / create_snapshot benchmarks
// ---------------------------------------------------------------------------

fn bench_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot");

    let small_tree = make_tree(3, 3); // ~40 nodes
    let medium_tree = make_tree(4, 4); // ~341 nodes

    group.bench_function("create_small_interactive_only", |b| {
        b.iter(|| snapshot::create_snapshot(black_box(&small_tree), Some("TestApp"), true, false));
    });

    group.bench_function("create_small_all_nodes", |b| {
        b.iter(|| snapshot::create_snapshot(black_box(&small_tree), Some("TestApp"), false, false));
    });

    group.bench_function("create_small_compact", |b| {
        b.iter(|| snapshot::create_snapshot(black_box(&small_tree), Some("TestApp"), false, true));
    });

    group.bench_function("create_medium_interactive_only", |b| {
        b.iter(|| snapshot::create_snapshot(black_box(&medium_tree), Some("TestApp"), true, false));
    });

    group.bench_function("create_medium_all_nodes", |b| {
        b.iter(|| {
            snapshot::create_snapshot(black_box(&medium_tree), Some("TestApp"), false, false)
        });
    });

    group.bench_function("create_medium_compact", |b| {
        b.iter(|| snapshot::create_snapshot(black_box(&medium_tree), Some("TestApp"), false, true));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion harness
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_selector_dsl, bench_snapshot);
criterion_main!(benches);
