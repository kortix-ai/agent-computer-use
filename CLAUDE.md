# agent-computer-use

Computer use CLI for AI agents. Built with Rust, distributed via npm.

## Repo structure

```
agent-computer-use/
├── cli/                Rust CLI (Cargo workspace)
│   ├── src/            Main binary, handlers, actions, snapshot, wait, DSL parser
│   └── crates/
│       ├── agent-computer-use-core/  Platform trait, AccessibilityNode, Selector, element utils
│       ├── agent-computer-use-macos/ macOS backend (AXUIElement, CGEvent, batch attribute fetch)
│       ├── agent-computer-use-cdp/   CDP bridge for Electron apps (WebSocket, DOM walker, JS interaction)
│       ├── agent-computer-use-linux/ Linux backend (AT-SPI2)
│       ├── agent-computer-use-windows/ Windows backend (UIAutomation)
│       └── agent-computer-use-mcp/   MCP server wrapper
├── docs/               Next.js docs site
├── benchmark/          E2E benchmarks, stress tests, comparison
├── bin/                JS wrapper + precompiled binaries
├── docker/             Cross-compilation Dockerfiles
├── scripts/            Setup, build, bench scripts
└── .github/            CI/CD workflows
```

## Quick reference

```bash
pnpm setup              # first-time setup (checks deps, builds, installs)
pnpm build              # build CLI (release)
pnpm test               # run Rust tests
pnpm lint               # cargo fmt --check + clippy
pnpm format             # auto-format Rust + JS/TS
pnpm dev                # start docs dev server
pnpm check              # lint + test (pre-commit)
pnpm bench              # criterion micro-benchmarks
pnpm bench:e2e          # real-world app benchmarks
pnpm bench:stress       # reliability stress test
pnpm bench:diff         # compare benchmark runs
```

## Working on the CLI

Rust code lives in `cli/`. The workspace has:

- `agent-computer-use` (crate) — produces the `agent-cu` binary, CLI args, handlers
- `agent-computer-use-core` — platform-agnostic types (Platform trait, AccessibilityNode, Selector, element utilities)
- `agent-computer-use-macos` — macOS backend (AXUIElement, CGEvent, batch `AXUIElementCopyMultipleAttributeValues`)
- `agent-computer-use-cdp` — CDP bridge (auto-detects Electron, auto-relaunches with debug port, WebSocket connection caching)

Key modules in `cli/src/`:

- `actions.rs` — click, type, drag with CDP-first routing, AXPress-first for native
- `snapshot.rs` — ref assignment, path-based caching, CDP element tagging
- `wait.rs` — polling, chain resolution, smart ranking, path resolution
- `selector_dsl.rs` — selector DSL parser (supports `id~=`, `name~=`, `css=`, `>>` chains, `index=N`)

Key modules in `cli/crates/agent-computer-use-cdp/src/`:

- `connection.rs` — WebSocket CDP client with URL caching (~15ms reconnect)
- `dom.rs` — JS DOM walker, element tagging (`data-acu`), click/type/scroll/key via JS
- `detect.rs` — Electron detection (bundle check + process args), auto-relaunch with CDP
- `lib.rs` — `ElectronAwarePlatform` decorator that intercepts Platform methods for CDP apps

CLI design:

- `-a` flag for app everywhere (no positional app args)
- `-s` flag for selector on `type`
- `click` handles both AXPress and coordinate fallback (no separate `press` command)
- `--cdp`, `--cdp-port`, `--no-cdp` are power-user overrides (CDP is automatic)
- All output is JSON by default

Run `cd cli && cargo build` for a debug build. Tests: `cargo test`.

## Working on docs

```bash
cd docs && npm run dev
```

Next.js + Tailwind + shadcn. Pages in `docs/src/app/`. Components in `docs/src/components/`.

## Before committing

Husky pre-commit hook runs automatically:

- Rust: `cargo fmt --check` + `cargo clippy -- -D warnings`
- If Rust files changed: `cargo test`

Fix formatting with `pnpm format`.
