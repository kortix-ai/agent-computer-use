use agent_computer_use_core::Platform;
use clap::Parser;
use serde::Serialize;
use std::time::Duration;

use crate::cli::args::Cli;
use crate::cli::handlers;
use crate::cli::output::{Output, RunError};

#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub command: Vec<String>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn execute_batch<'a>(
    platform: &'a dyn Platform,
    output: &'a Output,
    timeout: Duration,
    bail: bool,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = agent_computer_use_core::Result<Vec<BatchResult>>> + 'a>,
> {
    Box::pin(async move { execute_batch_inner(platform, output, timeout, bail).await })
}

async fn execute_batch_inner(
    platform: &dyn Platform,
    output: &Output,
    timeout: Duration,
    bail: bool,
) -> agent_computer_use_core::Result<Vec<BatchResult>> {
    let stdin =
        std::io::read_to_string(std::io::stdin()).map_err(agent_computer_use_core::Error::Io)?;
    let commands: Vec<Vec<String>> = serde_json::from_str(&stdin).map_err(|e| {
        agent_computer_use_core::Error::PlatformError {
            message: format!("invalid batch JSON: {e} — expected [[\"cmd\", \"arg\"], ...]"),
        }
    })?;

    let mut results = Vec::new();

    for cmd_args in &commands {
        if cmd_args.is_empty() {
            continue;
        }

        let mut full_args = vec!["agent-computer-use".to_string()];
        full_args.extend(cmd_args.clone());

        let parsed = match Cli::try_parse_from(&full_args) {
            Ok(cli) => cli,
            Err(e) => {
                let result = BatchResult {
                    command: cmd_args.clone(),
                    success: false,
                    error: Some(format!("parse error: {e}")),
                };
                if bail {
                    results.push(result);
                    return Ok(results);
                }
                results.push(result);
                continue;
            }
        };

        let cmd_timeout = Duration::from_secs_f64(parsed.timeout).min(timeout);

        match handlers::run(parsed.command, platform, output, cmd_timeout).await {
            Ok(()) => {
                results.push(BatchResult {
                    command: cmd_args.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(RunError::Core(e)) => {
                let result = BatchResult {
                    command: cmd_args.clone(),
                    success: false,
                    error: Some(e.to_string()),
                };
                if bail {
                    results.push(result);
                    return Ok(results);
                }
                results.push(result);
            }
            Err(RunError::ExpectFailed { message, .. }) => {
                let result = BatchResult {
                    command: cmd_args.clone(),
                    success: false,
                    error: Some(message),
                };
                if bail {
                    results.push(result);
                    return Ok(results);
                }
                results.push(result);
            }
        }
    }

    Ok(results)
}
