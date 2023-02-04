//! Helper enums to easily obtain the scan code of a key.

#[cfg(target_os = "macos")]
mod mac_os;

#[cfg(not(target_os = "macos"))]
mod set_1;

#[cfg(not(target_os = "macos"))]
pub use set_1::QwertyScanCode;
