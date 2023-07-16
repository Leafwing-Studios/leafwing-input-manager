//! Helper enum to define scan codes on the QWERTY keyboard layout.

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// The key locations as defined by the keys on the QWERTY keyboard layout.
///
/// The [`u32`] representation of this enum are the `KeyboardEvent.code` values on Wasm.
/// See <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#value_of_keycode>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[repr(u32)]
pub enum QwertyScanCode {
    /// The location of the `1` key on the QWERTY keyboard layout.
    Key1 = 0x31,
    /// The location of the `2` key on the QWERTY keyboard layout.
    Key2 = 0x32,
    /// The location of the `3` key on the QWERTY keyboard layout.
    Key3 = 0x33,
    /// The location of the `4` key on the QWERTY keyboard layout.
    Key4 = 0x34,
    /// The location of the `5` key on the QWERTY keyboard layout.
    Key5 = 0x35,
    /// The location of the `6` key on the QWERTY keyboard layout.
    Key6 = 0x36,
    /// The location of the `7` key on the QWERTY keyboard layout.
    Key7 = 0x37,
    /// The location of the `8` key on the QWERTY keyboard layout.
    Key8 = 0x38,
    /// The location of the `9` key on the QWERTY keyboard layout.
    Key9 = 0x39,
    /// The location of the `0` key on the QWERTY keyboard layout.
    Key0 = 0x30,
    /// The location of the `A`  key on the QWERTY keyboard layout.
    A = 0x41,
    /// The location of the `B` key on the QWERTY keyboard layout.
    B = 0x42,
    /// The location of the `C` key on the QWERTY keyboard layout.
    C = 0x43,
    /// The location of the `D`  key on the QWERTY keyboard layout.
    D = 0x44,
    /// The location of the `E`  key on the QWERTY keyboard layout.
    E = 0x45,
    /// The location of the `F`  key on the QWERTY keyboard layout.
    F = 0x46,
    /// The location of the `G`  key on the QWERTY keyboard layout.
    G = 0x47,
    /// The location of the `H`  key on the QWERTY keyboard layout.
    H = 0x48,
    /// The location of the `I`  key on the QWERTY keyboard layout.
    I = 0x49,
    /// The location of the `J`  key on the QWERTY keyboard layout.
    J = 0x4a,
    /// The location of the `K`  key on the QWERTY keyboard layout.
    K = 0x4b,
    /// The location of the `L`  key on the QWERTY keyboard layout.
    L = 0x4c,
    /// The location of the `M` key on the QWERTY keyboard layout.
    M = 0x4d,
    /// The location of the `N` key on the QWERTY keyboard layout.
    N = 0x4e,
    /// The location of the `O`  key on the QWERTY keyboard layout.
    O = 0x4f,
    /// The location of the `P`  key on the QWERTY keyboard layout.
    P = 0x50,
    /// The location of the `Q`  key on the QWERTY keyboard layout.
    Q = 0x51,
    /// The location of the `R`  key on the QWERTY keyboard layout.
    R = 0x52,
    /// The location of the `S`  key on the QWERTY keyboard layout.
    S = 0x53,
    /// The location of the `T`  key on the QWERTY keyboard layout.
    T = 0x54,
    /// The location of the `U`  key on the QWERTY keyboard layout.
    U = 0x55,
    /// The location of the `V key on the QWERTY keyboard layout.
    V = 0x56,
    /// The location of the `W`  key on the QWERTY keyboard layout.
    W = 0x57,
    /// The location of the `X` key on the QWERTY keyboard layout.
    X = 0x58,
    /// The location of the `Y`  key on the QWERTY keyboard layout.
    Y = 0x59,
    /// The location of the `Z` key on the QWERTY keyboard layout.
    Z = 0x5a,
    /// The location of the `,` key on the QWERTY keyboard layout.
    Comma = 0xbc,
    /// The location of the `.` key on the QWERTY keyboard layout.
    Period = 0xbe,
    /// The location of the `;`  key on the QWERTY keyboard layout.
    SemiColon = 0xba,
    /// The location of the `'`  key on the QWERTY keyboard layout.
    Apostrophe = 0xde,
    /// The location of the `[`  key on the QWERTY keyboard layout.
    BracketLeft = 0xdb,
    /// The location of the `]`  key on the QWERTY keyboard layout.
    BracketRight = 0xdd,
    /// The location of the `` ` `` key on the QWERTY keyboard layout.
    Backtick = 0xc0,
    /// The location of the `\` on the QWERTY keyboard layout.
    Backslash = 0xdc,
    /// The location of the `-` key on the QWERTY keyboard layout.
    Minus = 0xbd,
    /// The location of the `=` key on the QWERTY keyboard layout.
    Equals = 0xbb,
    /// The location of the left Alt  key on the QWERTY keyboard layout.
    /// Maps to left Option key on Apple keyboards.
    AltLeft = 0x12,
    /// The location of the right Alt key on the QWERTY keyboard layout.
    /// Maps to right Option key on Apple keyboards.
    ///
    /// On Wasm, this scan code value maps to the AltGraph key on Linux, if available.
    /// Note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    AltRight = 0xe1,
    /// The location of the caps lock  key on the QWERTY keyboard layout.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    CapsLock = 0x14,
    /// The location of the left Control key on the QWERTY keyboard layout.
    ///
    /// On Wasm, the right Control key has the same scan code, see:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    ControlLeft = 0x11,
    /// The location of the left Windows key on the QWERTY keyboard layout.
    /// Maps to the Command key on Apple keyboards.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    SuperLeft = 0x5b,
    /// The location of the right Windows key on the QWERTY keyboard layout.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    SuperRight = 0x5c,
    /// The location of the left Shift key on the QWERTY keyboard layout.
    ///
    /// On Wasm, the right Shift key has the same scan code, see:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    ShiftLeft = 0x10,

    /// The location of the Menu key on the QWERTY keyboard layout.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    Menu = 0x5d,
    /// The location of the Enter/Return key on the QWERTY keyboard layout.
    Enter = 0x0d,
    /// The location of the Space key on the QWERTY keyboard layout.
    Space = 0x20,
    /// The location of the tabulator key on the QWERTY keyboard layout.
    Tab = 0x09,
    /// The location of the Delete key on the QWERTY keyboard layout.
    Delete = 0x2e,
    /// The location of the End key on the QWERTY keyboard layout.
    End = 0x23,
    // Help = 0x2d, Not available on a lot of platforms and the scan code varies a lot
    /// The location of the Home key on the QWERTY keyboard layout.
    Home = 0x24,
    /// The location of the Insert key on the QWERTY keyboard layout.
    /// Maps to the Help key on Apple keyboards.
    Insert = 0x2d,
    /// The location of the Page Down key on the QWERTY keyboard layout.
    PageDown = 0x22,
    /// The location of the Page Up key on the QWERTY keyboard layout.
    PageUp = 0x21,
    /// The location of the Arrow Down key on the QWERTY keyboard layout.
    Down = 0x28,
    /// The location of the Arrow Left key on the QWERTY keyboard layout.
    Left = 0x25,
    /// The location of the Arrow Right key on the QWERTY keyboard layout.
    Right = 0x27,
    /// The location of the Arrow Up key on the QWERTY keyboard layout.
    Up = 0x26,
    /// The location of the Escape/Esc key on the QWERTY keyboard layout.
    Escape = 0x1b,
    /// The location of the Snapshot / Print Screen key on the QWERTY keyboard layout.
    /// Maps to the `F13` key on Apple keyboards.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    Snapshot = 0x2c,
    /// The location of the Scroll / Scroll Lock key on the QWERTY keyboard layout.
    /// Maps to the `F14` key on Apple keyboards.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    Scroll = 0x91,
    /// The location of the Pause key on the QWERTY keyboard layout.
    /// Maps to the `F15` key on Apple keyboards.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#non-printable_keys_function_keys>
    Pause = 0x13,

    /// The location of the `F1` key on the QWERTY keyboard layout.
    F1 = 0x70,
    /// The location of the `F2` key on the QWERTY keyboard layout.
    F2 = 0x71,
    /// The location of the `F3` key on the QWERTY keyboard layout.
    F3 = 0x72,
    /// The location of the `F4` key on the QWERTY keyboard layout.
    F4 = 0x73,
    /// The location of the `F5` key on the QWERTY keyboard layout.
    F5 = 0x74,
    /// The location of the `F6` key on the QWERTY keyboard layout.
    F6 = 0x75,
    /// The location of the `F7` key on the QWERTY keyboard layout.
    F7 = 0x76,
    /// The location of the `F8` key on the QWERTY keyboard layout.
    F8 = 0x77,
    /// The location of the `F9` key on the QWERTY keyboard layout.
    F9 = 0x78,
    /// The location of the `F10` key on the QWERTY keyboard layout.
    F10 = 0x79,
    /// The location of the `F11` key on the QWERTY keyboard layout.
    F11 = 0x7a,
    /// The location of the `F12` key on the QWERTY keyboard layout.
    F12 = 0x7b,
    // F13-F24: Ignoring for now as I'm not sure what they are equivalent to on Windows
    /// The location of the Numlock key on the QWERTY keyboard layout.
    /// Maps to `NumpadClear` on Apple keyboards.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#numpad_keys>
    Numlock = 0x90,
    /// The location of the `0` key on the numpad of the QWERTY keyboard layout.
    Numpad0 = 0x06,
    /// The location of the `1` key on the numpad of the QWERTY keyboard layout.
    Numpad1 = 0x61,
    /// The location of the `2` key on the numpad of the QWERTY keyboard layout.
    Numpad2 = 0x62,
    /// The location of the `3` key on the numpad of the QWERTY keyboard layout.
    Numpad3 = 0x63,
    /// The location of the `4` key on the numpad of the QWERTY keyboard layout.
    Numpad4 = 0x64,
    /// The location of the `5` key on the numpad of the QWERTY keyboard layout.
    Numpad5 = 0x65,
    /// The location of the `6` key on the numpad of the QWERTY keyboard layout.
    Numpad6 = 0x66,
    /// The location of the `7` key on the numpad of the QWERTY keyboard layout.
    Numpad7 = 0x67,
    /// The location of the `8` key on the numpad of the QWERTY keyboard layout.
    Numpad8 = 0x68,
    /// The location of the `9` key on the numpad of the QWERTY keyboard layout.
    Numpad9 = 0x69,
    /// The location of the `+` key on the numpad of the QWERTY keyboard layout.
    NumpadAdd = 0x6b,
    // NumpadComma = 0xc2: No idea what the difference to NumpadDecimal is.
    /// The location of the `.` key on the numpad of the QWERTY keyboard layout.
    ///
    /// On Wasm, note the platform specific details here:
    /// <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode#numpad_keys>
    NumpadDecimal = 0x6e,
    /// The location of the `/` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadEquals` on Apple keyboards.
    NumpadDivide = 0x6f,
    // NumpadEnter = 0x0d, This is the same as `Enter` on Wasm
    // NumpadEqual = 0x0c: No idea what the difference to NumpadEnter is.
    /// The location of the `*` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadDivide` on Apple keyboards.
    NumpadMultiply = 0x6a,
    /// The location of the `*` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadMultiply` on Apple keyboards.
    NumpadSubtract = 0x6d,
}
