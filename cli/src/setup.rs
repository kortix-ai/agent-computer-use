use agent_computer_use_core::{Error, Result};
use crossterm::style::{Color, Stylize};
use inquire::{formatter::OptionFormatter, ui::RenderConfig, Select};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::cli::args::{SetupMode, SetupScope};

const ALLOW_RULE: &str = "Bash(agent-cu *)";

const ACCENT: Color = Color::Rgb {
    r: 180,
    g: 220,
    b: 70,
};

pub fn run(yes: bool, mode: Option<SetupMode>, scope: Option<SetupScope>) -> Result<()> {
    print_banner();

    let effective_mode = mode.or(if yes {
        Some(SetupMode::Unsupervised)
    } else {
        None
    });
    let effective_scope = scope.or(if yes { Some(SetupScope::Global) } else { None });

    let unsupervised = match effective_mode {
        Some(SetupMode::Unsupervised) => true,
        Some(SetupMode::Supervised) => false,
        None => select_mode()?,
    };

    if !unsupervised {
        print_line();
        println!(
            "  {} {}",
            "›".dark_grey(),
            "Supervised — Claude Code will prompt for each command.".dim()
        );
        print_line();
        println!();
        return Ok(());
    }

    let scope_global = match effective_scope {
        Some(SetupScope::Global) => true,
        Some(SetupScope::Project) => false,
        None => select_scope()?,
    };

    let settings_path = if scope_global {
        global_settings_path()?
    } else {
        project_settings_path()?
    };

    apply_allow_rule(&settings_path)
}

const BANNER: &[&str] = &[
    r" █████╗  ██████╗ ███████╗███╗   ██╗████████╗     ██████╗██╗   ██╗",
    r"██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝    ██╔════╝██║   ██║",
    r"███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║       ██║     ██║   ██║",
    r"██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║       ██║     ██║   ██║",
    r"██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║       ╚██████╗╚██████╔╝",
    r"╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝        ╚═════╝ ╚═════╝ ",
];

fn print_banner() {
    println!();
    for line in BANNER {
        println!("  {}", line.with(ACCENT).bold());
    }
    println!();
    println!(
        "  {}  {}  {}",
        "agent · computer · use".with(ACCENT).bold(),
        "—".dark_grey(),
        "Configure Claude Code to run smoothly".dim()
    );
    println!();
}

fn print_line() {
    println!("  {}", "─".repeat(60).dark_grey());
}

fn short_formatter() -> OptionFormatter<'static, &'static str> {
    &|o| {
        o.value
            .split(" — ")
            .next()
            .unwrap_or(o.value)
            .trim()
            .to_string()
    }
}

fn select_mode() -> Result<bool> {
    let options = vec![
        "Unsupervised — auto-approve all agent-cu commands",
        "Supervised — ask approval before each command",
    ];
    let choice = Select::new("How should agent-cu run in Claude Code?", options)
        .with_render_config(render_config())
        .with_formatter(short_formatter())
        .with_help_message("↑↓ to navigate · enter to select")
        .prompt()
        .map_err(prompt_err)?;
    Ok(choice.starts_with("Unsupervised"))
}

fn select_scope() -> Result<bool> {
    let options = vec![
        "Global — applies to every project",
        "Project — applies only to this directory",
    ];
    let choice = Select::new("Scope?", options)
        .with_render_config(render_config())
        .with_formatter(short_formatter())
        .with_help_message("↑↓ to navigate · enter to select")
        .prompt()
        .map_err(prompt_err)?;
    Ok(choice.starts_with("Global"))
}

fn render_config() -> RenderConfig<'static> {
    use inquire::ui::{Attributes, Color as IColor, StyleSheet, Styled};

    let accent = IColor::Rgb {
        r: 180,
        g: 220,
        b: 70,
    };

    RenderConfig {
        prompt_prefix: Styled::new("›").with_fg(accent),
        answered_prompt_prefix: Styled::new("✓").with_fg(accent),
        selected_option: Some(
            StyleSheet::new()
                .with_fg(accent)
                .with_attr(Attributes::BOLD),
        ),
        highlighted_option_prefix: Styled::new("❯").with_fg(accent),
        help_message: StyleSheet::new().with_fg(IColor::DarkGrey),
        ..Default::default()
    }
}

fn prompt_err(e: inquire::InquireError) -> Error {
    Error::PlatformError {
        message: format!("setup cancelled: {e}"),
    }
}

fn apply_allow_rule(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut json: Value = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        match serde_json::from_str::<Value>(&content) {
            Ok(v) if v.is_object() => v,
            Ok(_) => {
                return Err(Error::PlatformError {
                    message: format!(
                        "{} exists but is not a JSON object; refusing to overwrite",
                        path.display()
                    ),
                });
            }
            Err(e) => {
                return Err(Error::PlatformError {
                    message: format!("{} has invalid JSON ({e}); fix it first", path.display()),
                });
            }
        }
    } else {
        json!({})
    };

    let root = json.as_object_mut().expect("validated above");
    let permissions = root
        .entry("permissions".to_string())
        .or_insert_with(|| json!({}));
    let perms_obj = permissions
        .as_object_mut()
        .ok_or_else(|| Error::PlatformError {
            message: "'permissions' exists but isn't an object".into(),
        })?;
    let allow = perms_obj
        .entry("allow".to_string())
        .or_insert_with(|| json!([]));
    let allow_array = allow.as_array_mut().ok_or_else(|| Error::PlatformError {
        message: "'permissions.allow' exists but isn't an array".into(),
    })?;

    let rule = Value::String(ALLOW_RULE.into());
    let already_present = allow_array.contains(&rule);
    if !already_present {
        allow_array.push(rule);
        let pretty = serde_json::to_string_pretty(&json)?;
        std::fs::write(path, pretty.as_bytes())?;
    }

    print_summary(path, already_present);
    Ok(())
}

fn print_summary(path: &Path, already_present: bool) {
    let pretty_path = pretty_path(path);

    println!();
    if already_present {
        println!(
            "  {}  {}",
            "•".dark_grey(),
            "Bash(agent-cu *) — already present".dim()
        );
        println!("    {}", pretty_path.dark_grey());
        println!();
        println!(
            "  {} {}",
            "›".dark_grey(),
            "Already configured. Nothing to restart.".dim()
        );
    } else {
        println!(
            "  {}  {}  {}",
            "✓".with(ACCENT).bold(),
            "Bash(agent-cu *)".with(ACCENT).bold(),
            "added to permissions.allow".dim()
        );
        println!("    {}", pretty_path.dark_grey());
        println!();
        println!(
            "  {} {}",
            "›".with(ACCENT),
            "Restart Claude Code to pick up the change.".bold()
        );
    }
    println!();
}

/// Replace leading $HOME / %USERPROFILE% with ~ for display.
fn pretty_path(path: &Path) -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok();
    if let Some(home) = home {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.display().to_string()
}

fn global_settings_path() -> Result<PathBuf> {
    // $HOME on Unix/macOS, %USERPROFILE% on Windows.
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| Error::PlatformError {
            message: "could not locate home directory (HOME / USERPROFILE unset)".into(),
        })?;
    Ok(PathBuf::from(home).join(".claude").join("settings.json"))
}

fn project_settings_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".claude").join("settings.local.json"))
}
