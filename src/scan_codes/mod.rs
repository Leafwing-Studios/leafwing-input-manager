//! Helper enums to easily obtain the scan code of a key.

// WASM
#[cfg(target_family = "wasm")]
mod mac_os;
#[cfg(target_family = "wasm")]
pub use mac_os::QwertyScanCode;

// MacOs
#[cfg(all(target_os = "macos", not(target_family = "wasm")))]
mod mac_os;
#[cfg(all(target_os = "macos", not(target_family = "wasm")))]
pub use mac_os::QwertyScanCode;

// Everything else (mainly Windows and Linux)
#[cfg(all(not(target_family = "wasm"), not(target_os = "macos")))]
mod set_1;
#[cfg(all(not(target_family = "wasm"), not(target_os = "macos")))]
pub use set_1::QwertyScanCode;
