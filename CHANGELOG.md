# Changelog

## 0.1.0 (Unreleased)

### Features

- **Selector DSL** — `agent-click click 'role=button name="Submit"'` with `>>` chaining
- **Snapshot + refs** — `agent-click snapshot Calculator -i` assigns `@e1, @e2, ...` refs for instant targeting
- **Background operation** — AXPress clicks and AXValue typing without stealing focus
- **Smart disambiguation** — ranks visible/enabled/interactive elements over hidden ones
- **Auto-wait** — actions poll for elements with configurable timeout
- **`--expect`** — post-action verification (`agent-click click 'name="Login"' --expect 'name="Dashboard"'`)
- **Workflows** — `agent-click run workflow.yaml` for multi-step automation
- **Batch mode** — `echo '[["click","@e3"]]' | agent-click batch` for piped execution
- **TUI observer** — `agent-click observe Calculator` for live tree exploration
- **Stealth key sending** — background key events with sub-100ms app switching
- **Verbose mode** — `-v` flag for debug traces

### Commands

`tree`, `find`, `click`, `type`, `key`, `scroll`, `snapshot`, `wait-for`,
`ensure-text`, `open`, `run`, `batch`, `observe`, `screenshot`, `windows`,
`focused`, `text`, `apps`, `check-permissions`

### Platforms

- macOS (AXUIElement + CGEvent) — production ready
- Linux (AT-SPI2) — stub, contributions welcome
- Windows (UI Automation) — stub, contributions welcome
