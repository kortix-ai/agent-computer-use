#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[cfg(not(target_os = "macos"))]
compile_error!("agent-computer-use-macos only compiles on macOS");

pub mod ax;
pub mod input;
mod platform;

pub use platform::MacOSPlatform;
