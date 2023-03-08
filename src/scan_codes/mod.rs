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

// Linux
#[cfg(all(target_os = "linux", not(target_os = "macos")))]
mod linux;
#[cfg(all(target_os = "linux", not(target_os = "macos")))]
pub use linux::QwertyScanCode;

// Windows and other stuff using Set 1
#[cfg(all(
    not(target_family = "wasm"),
    not(target_os = "macos"),
    not(target_os = "linux")
))]
mod windows;
#[cfg(all(
    not(target_family = "wasm"),
    not(target_os = "macos"),
    not(target_os = "linux")
))]
pub use windows::QwertyScanCode;

impl From<QwertyScanCode> for ScanCode {
    fn from(value: QwertyScanCode) -> Self {
        ScanCode(value as u32)
    }
}
