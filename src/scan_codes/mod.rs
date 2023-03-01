//! Helper enums to easily obtain the scan code of a key.
use bevy::prelude::ScanCode;

// Wasm
#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
pub use wasm::QwertyScanCode;

// MacOs
#[cfg(all(target_os = "macos", not(target_family = "wasm")))]
mod mac_os;
#[cfg(all(target_os = "macos", not(target_family = "wasm")))]
pub use mac_os::QwertyScanCode;

// Everything else (mainly Windows and Linux)
#[cfg(all(not(target_family = "wasm"), not(target_os = "macos")))]
mod windows;
#[cfg(all(not(target_family = "wasm"), not(target_os = "macos")))]
pub use windows::QwertyScanCode;

impl From<QwertyScanCode> for ScanCode {
    fn from(value: QwertyScanCode) -> Self {
        ScanCode(value as u32)
    }
}
