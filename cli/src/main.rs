pub mod actions;
pub mod batch;
pub mod cli;
mod observe;
pub mod record;
mod selector_dsl;
pub mod snapshot;
pub mod wait;
pub mod workflow;

use agent_computer_use_core::Platform;
use clap::Parser;
use cli::output::{ExpectOutcome, ExpectResult, RunError};
use cli::{Cli, Output};
use std::time::Duration;

const EXIT_TIMEOUT: i32 = 2;
const EXIT_EXPECT_FAILED: i32 = 3;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::try_from_env("AGENT_CLICK_LOG")
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"))
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let output = Output::new(cli.human, cli.compact);
    let timeout = Duration::from_secs_f64(cli.timeout);
    let platform = create_platform(cli.cdp, cli.cdp_port, cli.no_cdp);

    match cli::handlers::run(cli.command, &*platform, &output, timeout).await {
        Ok(()) => {}
        Err(RunError::Core(e)) => {
            output.error(&e);
            match e {
                agent_computer_use_core::Error::Timeout { .. } => std::process::exit(EXIT_TIMEOUT),
                _ => std::process::exit(1),
            }
        }
        Err(RunError::ExpectFailed {
            action_result,
            message,
        }) => {
            output.print(&ExpectResult {
                success: action_result.success,
                message: action_result.message,
                expect: ExpectOutcome {
                    met: false,
                    message: Some(message),
                    element: None,
                },
            });
            std::process::exit(EXIT_EXPECT_FAILED);
        }
    }
}

fn create_platform(cdp: bool, cdp_port: Option<u16>, no_cdp: bool) -> Box<dyn Platform> {
    let native = create_native_platform();
    let cdp_config = agent_computer_use_cdp::CdpConfig {
        port: cdp_port,
        force: cdp,
        disabled: no_cdp,
    };
    Box::new(agent_computer_use_cdp::ElectronAwarePlatform::new(
        native, cdp_config,
    ))
}

fn create_native_platform() -> impl Platform {
    #[cfg(target_os = "macos")]
    {
        agent_computer_use_macos::MacOSPlatform::new()
    }
    #[cfg(target_os = "linux")]
    {
        agent_computer_use_linux::LinuxPlatform::new()
    }
    #[cfg(target_os = "windows")]
    {
        agent_computer_use_windows::WindowsPlatform::new()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        compile_error!("unsupported platform");
    }
}
