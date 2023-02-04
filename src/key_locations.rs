//! Helpers for locations of keys on the keyboards.

use bevy::prelude::ScanCode;

// TODO: Verify that this works on MacOS

/// The key locations as defined by the keys on the QWERTY keyboard layout.
///
/// The [`u32`] representation of this enum are the Set 1 scan codes of the corresponding keys.
/// See section 10.6 at <https://www.win.tue.nl/~aeb/linux/kbd/scancodes-10.html#scancodesets>.
#[repr(u32)]
pub enum QwertyKeyLocation {
    /// The location of the `` ` `` key on the QWERTY keyboard layout.
    Backtick = 0x29,
    /// The location of the `1` key on the QWERTY keyboard layout.
    Key1 = 0x02,
    /// The location of the `2` key on the QWERTY keyboard layout.
    Key2 = 0x03,
    /// The location of the `3` key on the QWERTY keyboard layout.
    Key3 = 0x04,
    /// The location of the `4` key on the QWERTY keyboard layout.
    Key4 = 0x05,
    /// The location of the `5` key on the QWERTY keyboard layout.
    Key5 = 0x06,
    /// The location of the `6` key on the QWERTY keyboard layout.
    Key6 = 0x07,
    /// The location of the `7` key on the QWERTY keyboard layout.
    Key7 = 0x08,
    /// The location of the `8` key on the QWERTY keyboard layout.
    Key8 = 0x09,
    /// The location of the `9` key on the QWERTY keyboard layout.
    Key9 = 0x0a,
    /// The location of the `0` key on the QWERTY keyboard layout.
    Key0 = 0x0b,
    /// The location of the `-` key on the QWERTY keyboard layout.
    Minus = 0x0c,
    /// The location of the `=` key on the QWERTY keyboard layout.
    Equals = 0x0d,
    /// The location of the back(space) key on the QWERTY keyboard layout.
    Backspace = 0x0e,
    /// The location of the tabulator key on the QWERTY keyboard layout.
    Tab = 0x0f,
    /// The location of the `Q`  key on the QWERTY keyboard layout.
    Q = 0x10,
    /// The location of the `W`  key on the QWERTY keyboard layout.
    W = 0x11,
    /// The location of the `E`  key on the QWERTY keyboard layout.
    E = 0x12,
    /// The location of the `R`  key on the QWERTY keyboard layout.
    R = 0x13,
    /// The location of the `T`  key on the QWERTY keyboard layout.
    T = 0x14,
    /// The location of the `Y`  key on the QWERTY keyboard layout.
    Y = 0x15,
    /// The location of the `U`  key on the QWERTY keyboard layout.
    U = 0x16,
    /// The location of the `I`  key on the QWERTY keyboard layout.
    I = 0x17,
    /// The location of the `O`  key on the QWERTY keyboard layout.
    O = 0x18,
    /// The location of the `P`  key on the QWERTY keyboard layout.
    P = 0x19,
    /// The location of the `[`  key on the QWERTY keyboard layout.
    LBracket = 0x1a,
    /// The location of the `]`  key on the QWERTY keyboard layout.
    RBracket = 0x1b,
    /// The location of the `\` on the QWERTY keyboard layout.
    Backslash = 0x2b,
    /// The location of the caps lock  key on the QWERTY keyboard layout.
    CapsLock = 0x3a,
    /// The location of the `A`  key on the QWERTY keyboard layout.
    A = 0x1e,
    /// The location of the `S`  key on the QWERTY keyboard layout.
    S = 0x1f,
    /// The location of the `D`  key on the QWERTY keyboard layout.
    D = 0x20,
    /// The location of the `F`  key on the QWERTY keyboard layout.
    F = 0x21,
    /// The location of the `G`  key on the QWERTY keyboard layout.
    G = 0x22,
    /// The location of the `H`  key on the QWERTY keyboard layout.
    H = 0x23,
    /// The location of the `J`  key on the QWERTY keyboard layout.
    J = 0x24,
    /// The location of the `K`  key on the QWERTY keyboard layout.
    K = 0x25,
    /// The location of the `L`  key on the QWERTY keyboard layout.
    L = 0x26,
    /// The location of the `;`  key on the QWERTY keyboard layout.
    SemiColon = 0x27,
    /// The location of the `'`  key on the QWERTY keyboard layout.
    Apostrophe = 0x28,
    /// A key not available on the US QWERTY layout.
    ///
    /// This is for example the `#` key on other layouts.
    NonUs1 = 0x00,
    /// The location of the enter  key on the QWERTY keyboard layout.
    Enter = 0x1c,
    /// The location of the left shift  key on the QWERTY keyboard layout.
    LShift = 0x2a,
    /// The location of the `Z`  key on the QWERTY keyboard layout.
    Z = 0x2c,
    /// The location of the `X`  key on the QWERTY keyboard layout.
    X = 0x2d,
    /// The location of the `C`  key on the QWERTY keyboard layout.
    C = 0x2e,
    /// The location of the `V`  key on the QWERTY keyboard layout.
    V = 0x2f,
    /// The location of the `B`  key on the QWERTY keyboard layout.
    B = 0x30,
    /// The location of the `N`  key on the QWERTY keyboard layout.
    N = 0x31,
    /// The location of the `M`  key on the QWERTY keyboard layout.
    M = 0x32,
    /// The location of the `,`  key on the QWERTY keyboard layout.
    Comma = 0x33,
    /// The location of the `.`  key on the QWERTY keyboard layout.
    Period = 0x34,
    /// The location of the `/`  key on the QWERTY keyboard layout.
    Slash = 0x35,
    /// The location of the right shift  key on the QWERTY keyboard layout.
    RShift = 0x36,
    /// The location of the left control  key on the QWERTY keyboard layout.
    LCtrl = 0x1d,
    /// The location of the left alt  key on the QWERTY keyboard layout.
    LAlt = 0x38,
    /// The location of the space  key on the QWERTY keyboard layout.
    Space = 0x39,
    /// The location of the right alt  key on the QWERTY keyboard layout.
    RAlt = 0xe0_e8,
    /// The location of the right control  key on the QWERTY keyboard layout.
    RCtrl = 0xe0_1d,
    /// The location of the insert  key on the QWERTY keyboard layout.
    Insert = 0xe0_52,
    /// The location of the delete  key on the QWERTY keyboard layout.
    Delete = 0xe0_53,
    /// The location of the home  key on the QWERTY keyboard layout.
    Home = 0xe0_47,
    /// The location of the end  key on the QWERTY keyboard layout.
    End = 0xe0_4f,
    /// The location of the page up  key on the QWERTY keyboard layout.
    PageUp = 0xe0_49,
    /// The location of the page down  key on the QWERTY keyboard layout.
    PageDown = 0xe0_51,
    /// The location of the left arrow  key on the QWERTY keyboard layout.
    Left = 0xe0_4b,
    /// The location of the arrow up  key on the QWERTY keyboard layout.
    Up = 0xe0_48,
    /// The location of the arrow down  key on the QWERTY keyboard layout.
    Down = 0xe0_50,
    /// The location of the right arrow  key on the QWERTY keyboard layout.
    Right = 0xe0_4d,
    // FIXME: Add Numpad
}

impl From<QwertyKeyLocation> for ScanCode {
    fn from(value: QwertyKeyLocation) -> Self {
        ScanCode(value as u32)
    }
}
