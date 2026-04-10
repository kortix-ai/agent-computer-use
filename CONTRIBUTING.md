# Contributing to agent-click

Thanks for your interest! agent-click aims to be the universal CLI for desktop
automation — and that's only possible with community help.

## Where to contribute

### High-impact areas

- **Linux backend** (`crates/agent-click-linux/`) — Implement AT-SPI2 via D-Bus
  using the `zbus` and `atspi` crates. The `Platform` trait is defined in
  `crates/agent-click-core/src/platform.rs`.

- **Windows backend** (`crates/agent-click-windows/`) — Implement UI Automation
  using `windows-rs`. Focus on `IUIAutomationInvokePattern::Invoke()` for
  background clicks (agent-click's killer feature).

- **MCP server** (`crates/agent-click-mcp/`) — Expose agent-click as an MCP tool server
  so AI agents can use it directly.

### Good first issues

- Add missing role mappings in `crates/agent-click-macos/src/ax.rs`
- Add shell completions (bash, zsh, fish) via `clap_complete`
- Improve error messages with actionable suggestions
- Add tests for `src/wait.rs` ranking logic
- Add tests for `src/workflow.rs` YAML parsing

## Architecture

```
src/
├── main.rs               # Entry point (30 lines)
├── cli/                  # CLI layer
│   ├── args.rs           # Clap definitions (purely declarative)
│   ├── handlers.rs       # Command dispatch (thin, calls actions)
│   └── output.rs         # JSON/human output formatting
├── actions.rs            # Shared action lego blocks (click, type, find)
├── snapshot.rs           # Ref assignment + caching
├── selector_dsl.rs       # Selector DSL parser
├── wait.rs               # Polling + smart element ranking
├── workflow.rs           # YAML workflow engine
├── batch.rs              # Batch command executor
└── observe.rs            # TUI tree explorer

crates/
├── agent-click-core/            # Platform-agnostic contract
├── agent-click-macos/           # macOS: AXUIElement + CGEvent
├── agent-click-linux/           # Linux: AT-SPI2 (stub)
├── agent-click-windows/         # Windows: UIA (stub)
└── agent-click-mcp/             # MCP server (stub)
```

**Key principle:** All platform-specific code lives in backend crates.
The `src/` directory and `agent-click-core` are platform-agnostic.

## Development setup

```bash
git clone https://github.com/kortix-ai/agent-click
cd agent-click
cargo build
cargo test
cargo run -- snapshot Calculator -i
```

### Requirements

- Rust 1.75+
- macOS: Xcode Command Line Tools + Accessibility permissions
- Linux: `libdbus-1-dev` (for AT-SPI2)
- Windows: Visual Studio Build Tools

### Accessibility permissions

```
System Settings > Privacy & Security > Accessibility > [your terminal]
```

## Code style

- `cargo fmt` before committing
- `cargo clippy` with zero warnings
- Doc comments (`///`) on all public items, using `# Arguments` / `# Returns` sections
- No inline comments — use doc comments above functions instead
- Platform-specific code stays in backend crates, never in `src/`

## Pull requests

1. Fork and create a feature branch
2. Make changes
3. `cargo fmt && cargo clippy && cargo test`
4. Open a PR against `main`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
