#[cfg(target_os = "windows")]
pub mod input;
#[cfg(target_os = "windows")]
mod platform;
#[cfg(target_os = "windows")]
mod uia;

#[cfg(target_os = "windows")]
pub use platform::WindowsPlatform;

#[cfg(not(target_os = "windows"))]
mod platform;

#[cfg(not(target_os = "windows"))]
pub use platform::WindowsPlatform;
