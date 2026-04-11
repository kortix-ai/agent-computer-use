#[cfg(target_os = "linux")]
mod atspi;
#[cfg(target_os = "linux")]
pub mod input;

mod platform;

pub use platform::LinuxPlatform;
