use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agent-click",
    version,
    about = "Computer use for AI agents",
    long_about = "agent-click — control any desktop app from the command line via the accessibility tree.\n\n\
                  Click buttons, type text, read screens — all without stealing focus.\n\
                  Built for AI agents. Works for humans too.\n\n\
                  All output is JSON by default (machine-readable). Use --human for pretty output.",
    after_help = "Quick start:\n  \
                  agent-click apps                              # list running apps\n  \
                  agent-click snapshot Calculator -i -c          # interactive elements with refs\n  \
                  agent-click click @e5                          # click by ref\n  \
                  agent-click type @e3 \"hello\"                   # type into element\n  \
                  agent-click key cmd+c --app Safari             # press key combo\n\n\
                  Docs: https://github.com/kortix-ai/agent-click"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Human-readable output instead of JSON
    #[arg(long, global = true)]
    pub human: bool,

    /// Compact single-line JSON output
    #[arg(long, global = true)]
    pub compact: bool,

    /// Global timeout in seconds [default: 5]
    #[arg(long, global = true, default_value = "5")]
    pub timeout: f64,

    /// Show debug traces (selector resolution, timing)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Force CDP (Chrome DevTools Protocol) mode
    #[arg(long, global = true)]
    pub cdp: bool,

    /// CDP port override
    #[arg(long, global = true)]
    pub cdp_port: Option<u16>,

    /// Disable CDP auto-detection
    #[arg(long, global = true)]
    pub no_cdp: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Dump the accessibility tree for an app
    ///
    /// Shows all UI elements in a structured tree format.
    /// Use `snapshot` instead for AI agent workflows.
    #[command(alias = "t")]
    Tree {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Max tree depth to traverse
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Find elements matching a selector
    ///
    /// Returns all matching elements as a JSON array.
    ///
    /// Examples:
    ///   agent-click find 'role=button' --app Calculator
    ///   agent-click find 'name~="Submit"' --app Safari
    ///   agent-click find 'id="login-btn"' -d 5
    #[command(alias = "f")]
    Find {
        /// Selector DSL expression or @ref
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Max tree depth to search
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Click an element (background, no focus steal)
    ///
    /// Tries AXPress first (no mouse, no focus steal).
    /// Falls back to coordinate click if AXPress is unsupported.
    ///
    /// Examples:
    ///   agent-click click @e5
    ///   agent-click click 'name="Login"' --app Safari
    ///   agent-click click @e5 --count 2        # double-click
    ///   agent-click click @e5 --expect 'name="Dashboard"'
    #[command(alias = "c")]
    Click {
        /// Selector DSL expression or @ref
        selector: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Click at X coordinate
        #[arg(long)]
        x: Option<f64>,
        /// Click at Y coordinate
        #[arg(long)]
        y: Option<f64>,
        /// Mouse button: left (default), right, middle
        #[arg(long)]
        button: Option<String>,
        /// Number of clicks (2 = double-click)
        #[arg(long)]
        count: Option<u32>,
        /// Verify this element appears after clicking
        #[arg(long)]
        expect: Option<String>,
    },

    /// Clear a field and type text into it
    ///
    /// With --selector: uses AXValue (reliable, no focus steal).
    /// Without: simulates keypresses into the focused app.
    ///
    /// Examples:
    ///   agent-click type "hello" -s @e3           # type into element (AXValue)
    ///   agent-click type "hello" --app Safari     # keyboard simulation
    ///   agent-click type "more" -s @e3 --append   # append without clearing
    #[command(name = "type")]
    Type {
        /// Text to type
        text: String,
        /// Target element (selector DSL or @ref)
        #[arg(short, long)]
        selector: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Press Return/Enter after typing
        #[arg(long)]
        submit: bool,
        /// Append text instead of replacing
        #[arg(long)]
        append: bool,
        /// Verify this element appears after typing
        #[arg(long)]
        expect: Option<String>,
    },

    /// Press a key or key combination
    ///
    /// Examples:
    ///   agent-click key Return --app Calculator
    ///   agent-click key cmd+c --app TextEdit
    ///   agent-click key cmd+shift+p --app "VS Code"
    #[command(alias = "k")]
    Key {
        /// Key expression (e.g., Return, cmd+c, ctrl+shift+p)
        key: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Verify this element appears after key press
        #[arg(long)]
        expect: Option<String>,
    },

    /// Scroll an element into view using AXScrollToVisible
    ///
    /// Examples:
    ///   agent-click scroll-to @e42
    ///   agent-click scroll-to 'name="Submit"' --app Safari
    #[command(name = "scroll-to")]
    ScrollTo {
        /// Element to scroll into view (selector DSL or @ref)
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Scroll within an app
    ///
    /// Examples:
    ///   agent-click scroll down --app Music
    ///   agent-click scroll down --amount 10 --app Music
    ///   agent-click scroll down --at 'name="List"' --app Music
    #[command(alias = "s")]
    Scroll {
        /// Direction: up, down, left, right
        direction: String,
        /// Number of scroll ticks [default: 3]
        #[arg(long)]
        amount: Option<u32>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Position mouse at this element before scrolling
        #[arg(long, name = "at")]
        at_selector: Option<String>,
        /// Verify this element appears after scrolling
        #[arg(long)]
        expect: Option<String>,
    },

    /// Drag from one element/position to another
    Drag {
        /// Source element (selector DSL or @ref)
        from: Option<String>,
        /// Destination element (selector DSL or @ref)
        to: Option<String>,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Source X coordinate (if not using selector)
        #[arg(long)]
        from_x: Option<f64>,
        /// Source Y coordinate (if not using selector)
        #[arg(long)]
        from_y: Option<f64>,
        /// Destination X coordinate (if not using selector)
        #[arg(long)]
        to_x: Option<f64>,
        /// Destination Y coordinate (if not using selector)
        #[arg(long)]
        to_y: Option<f64>,
    },

    /// Wait for an element to appear
    ///
    /// Polls the accessibility tree at a regular interval.
    /// Returns the element when found, or errors on timeout.
    ///
    /// Examples:
    ///   agent-click wait-for 'name="Dashboard"'
    ///   agent-click wait-for 'role=button' --interval 500
    #[command(name = "wait-for", alias = "wait")]
    WaitFor {
        /// Element to wait for (selector DSL)
        selector: String,
        /// Poll interval in milliseconds [default: 200]
        #[arg(long, default_value = "200")]
        interval: u64,
    },

    /// Set text only if it differs from the current value
    ///
    /// Idempotent: won't type if the value already matches.
    ///
    /// Examples:
    ///   agent-click ensure-text @e3 "hello@example.com"
    #[command(name = "ensure-text")]
    EnsureText {
        /// Target element (selector DSL or @ref)
        selector: String,
        /// Expected text value
        text: String,
    },

    /// Get an element's value, name, role, and position
    ///
    /// Lighter than `snapshot` — reads a single element.
    ///
    /// Examples:
    ///   agent-click get-value @e3
    ///   agent-click gv 'name="Email"' --app Safari
    #[command(name = "get-value", alias = "gv")]
    GetValue {
        /// Target element (selector DSL or @ref)
        selector: String,
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Launch an application
    ///
    /// Examples:
    ///   agent-click open Safari --wait
    ///   agent-click open Calculator
    Open {
        /// Application name
        app: String,
        /// Wait for the app window to appear
        #[arg(long)]
        wait: bool,
    },

    /// Execute a YAML workflow file
    ///
    /// Runs multi-step automation from a YAML file.
    /// See examples/ folder for workflow templates.
    ///
    /// Examples:
    ///   agent-click run examples/calculator.yaml
    ///   agent-click run workflow.yaml --app Calculator
    ///   agent-click run workflow.yaml --dry-run
    #[command(alias = "r")]
    Run {
        /// Path to YAML workflow file
        file: String,
        /// Override the default app in the workflow
        #[arg(short, long)]
        app: Option<String>,
        /// Validate without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Interactive TUI tree explorer
    ///
    /// Navigate the accessibility tree in real time.
    /// Keys: j/k navigate, Enter expand, / search, y copy, q quit
    Observe {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Max tree depth [default: 10]
        #[arg(short, long, default_value = "10")]
        depth: u32,
        /// Refresh interval in seconds [default: 2.0]
        #[arg(long, default_value = "2.0")]
        refresh: f64,
    },

    /// Snapshot the accessibility tree with refs
    ///
    /// The primary command for AI agent workflows. Returns a
    /// compact tree with @refs (e.g., @e1, @e2) for each element.
    /// Use refs with other commands: `agent-click click @e5`
    ///
    /// Examples:
    ///   agent-click snapshot --app Calculator -i -c
    ///   agent-click snapshot --app Music -i -c -d 8
    #[command(alias = "snap")]
    Snapshot {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Max tree depth
        #[arg(short, long)]
        depth: Option<u32>,
        /// Show only interactive elements (buttons, inputs, links)
        #[arg(short, long)]
        interactive: bool,
        /// Remove empty structural elements
        #[arg(short, long)]
        compact: bool,
    },

    /// Execute multiple commands from stdin
    ///
    /// Pipe a JSON array of commands to avoid per-command startup cost.
    ///
    /// Examples:
    ///   echo '[["click","@e5"],["click","@e8"]]' | agent-click batch
    ///   echo '[["click","@e1"]]' | agent-click batch --bail
    Batch {
        /// Stop on first error
        #[arg(long)]
        bail: bool,
    },

    /// Take a screenshot
    ///
    /// Examples:
    ///   agent-click screenshot
    ///   agent-click screenshot --path /tmp/screen.png
    ///   agent-click screenshot --app Calculator
    Screenshot {
        /// Save path (auto-generated if omitted)
        #[arg(long)]
        path: Option<String>,
        /// Capture only this app's window
        #[arg(short, long)]
        app: Option<String>,
    },

    /// List windows for an app
    Windows {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
    },

    /// Move an app's window to a position
    #[command(name = "move-window")]
    MoveWindow {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// X position
        #[arg(long)]
        x: f64,
        /// Y position
        #[arg(long)]
        y: f64,
    },

    /// Resize an app's window
    #[command(name = "resize-window")]
    ResizeWindow {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
        /// Width
        #[arg(long)]
        width: f64,
        /// Height
        #[arg(long)]
        height: f64,
    },

    /// Get the currently focused element
    Focused,

    /// Get all visible text from an app
    ///
    /// Traverses the full tree — use `get-value` for specific elements.
    Text {
        /// Target application name
        #[arg(short, long)]
        app: Option<String>,
    },

    /// List running applications
    Apps,

    /// Check if accessibility permissions are granted
    #[command(name = "check-permissions", alias = "check")]
    CheckPermissions,

    /// Generate shell completions
    ///
    /// Output shell completions to stdout.
    ///
    /// Setup:
    ///   agent-click completions bash > /etc/bash_completion.d/agent-click
    ///   agent-click completions zsh  > "${fpath[1]}/_agent-click"
    ///   agent-click completions fish > ~/.config/fish/completions/agent-click.fish
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
