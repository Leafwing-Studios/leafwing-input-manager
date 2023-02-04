//! Helpers for locations of keys on the keyboards.

/// The key locations as defined by the keys on the QWERTY keyboard layout.
/// See <https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#scan-codes>
#[repr(u32)]
pub enum QwertyKeyLocation {
    /// The location of the `` ` `` key on the QWERTY keyboard layout.
    Backtick = 1,
    /// The location of the `1` key on the QWERTY keyboard layout.
    Key1 = 2,
    /// The location of the `2` key on the QWERTY keyboard layout.
    Key2 = 3,
    /// The location of the `3` key on the QWERTY keyboard layout.
    Key3 = 4,
    /// The location of the `4` key on the QWERTY keyboard layout.
    Key4 = 5,
    /// The location of the `5` key on the QWERTY keyboard layout.
    Key5 = 6,
    /// The location of the `6` key on the QWERTY keyboard layout.
    Key6 = 7,
    /// The location of the `7` key on the QWERTY keyboard layout.
    Key7 = 8,
    /// The location of the `8` key on the QWERTY keyboard layout.
    Key8 = 9,
    /// The location of the `9` key on the QWERTY keyboard layout.
    Key9 = 10,
    /// The location of the `0` key on the QWERTY keyboard layout.
    Key0 = 11,
    /// The location of the `-` key on the QWERTY keyboard layout.
    Minus = 12,
    /// The location of the `=` key on the QWERTY keyboard layout.
    Equals = 13,
    /// The location of the `\` key on the QWERTY keyboard layout.
    Backslash = 14,
    /// The location of the back(space) key on the QWERTY keyboard layout.
    Back = 15,
    /// The location of the tabulator key on the QWERTY keyboard layout.
    Tab = 16,
    /// The location of the `Q`  key on the QWERTY keyboard layout.
    Q = 17,
    /// The location of the `W`  key on the QWERTY keyboard layout.
    W = 18,
    /// The location of the `E`  key on the QWERTY keyboard layout.
    E = 19,
    /// The location of the `R`  key on the QWERTY keyboard layout.
    R = 20,
    /// The location of the `T`  key on the QWERTY keyboard layout.
    T = 21,
    /// The location of the `Y`  key on the QWERTY keyboard layout.
    Y = 22,
    /// The location of the `U`  key on the QWERTY keyboard layout.
    U = 23,
    /// The location of the `I`  key on the QWERTY keyboard layout.
    I = 24,
    /// The location of the `O`  key on the QWERTY keyboard layout.
    O = 25,
    /// The location of the `P`  key on the QWERTY keyboard layout.
    P = 26,
    /// The location of the `[`  key on the QWERTY keyboard layout.
    LBracket = 27,
    /// The location of the `]`  key on the QWERTY keyboard layout.
    RBracket = 28,
    /// The location of the enter  key on the QWERTY keyboard layout.
    Enter = 29,
    /// The location of the caps lock  key on the QWERTY keyboard layout.
    CapsLock = 30,
    /// The location of the `A`  key on the QWERTY keyboard layout.
    A = 31,
    /// The location of the `S`  key on the QWERTY keyboard layout.
    S = 32,
    /// The location of the `D`  key on the QWERTY keyboard layout.
    D = 33,
    /// The location of the `F`  key on the QWERTY keyboard layout.
    F = 34,
    /// The location of the `G`  key on the QWERTY keyboard layout.
    G = 35,
    /// The location of the `H`  key on the QWERTY keyboard layout.
    H = 36,
    /// The location of the `J`  key on the QWERTY keyboard layout.
    J = 37,
    /// The location of the `K`  key on the QWERTY keyboard layout.
    K = 38,
    /// The location of the `L`  key on the QWERTY keyboard layout.
    L = 39,
    /// The location of the `;`  key on the QWERTY keyboard layout.
    SemiColon = 40,
    /// The location of the `'`  key on the QWERTY keyboard layout.
    Apostrophe = 41,

    /// The location of the left shift  key on the QWERTY keyboard layout.
    LShift = 44,
    /// The location of the `Z`  key on the QWERTY keyboard layout.
    Z = 45,
    /// The location of the `X`  key on the QWERTY keyboard layout.
    X = 46,
    /// The location of the `C`  key on the QWERTY keyboard layout.
    C = 47,
    /// The location of the `V`  key on the QWERTY keyboard layout.
    V = 48,
    /// The location of the `B`  key on the QWERTY keyboard layout.
    B = 49,
    /// The location of the `N`  key on the QWERTY keyboard layout.
    N = 50,
    /// The location of the `M`  key on the QWERTY keyboard layout.
    M = 51,
    /// The location of the `,`  key on the QWERTY keyboard layout.
    Comma = 52,
    /// The location of the `.`  key on the QWERTY keyboard layout.
    Period = 53,
    /// The location of the `/`  key on the QWERTY keyboard layout.
    Slash = 54,

    /// The location of the left control  key on the QWERTY keyboard layout.
    LCtrl = 58,

    /// The location of the left alt  key on the QWERTY keyboard layout.
    LAlt = 60,
    /// The location of the space  key on the QWERTY keyboard layout.
    Space = 61,
    /// The location of the right alt  key on the QWERTY keyboard layout.
    RAlt = 62,

    /// The location of the right control  key on the QWERTY keyboard layout.
    RCtrl = 64,

    /// The location of the insert  key on the QWERTY keyboard layout.
    Insert = 75,
    /// The location of the delete  key on the QWERTY keyboard layout.
    Delete = 76,

    /// The location of the left arrow  key on the QWERTY keyboard layout.
    ArrowLeft = 79,
    /// The location of the home  key on the QWERTY keyboard layout.
    Home = 80,
    /// The location of the end  key on the QWERTY keyboard layout.
    End = 81,

    /// The location of the arrow up  key on the QWERTY keyboard layout.
    ArrowUp = 83,
    /// The location of the arrow down  key on the QWERTY keyboard layout.
    ArrowDown = 84,
    /// The location of the page up  key on the QWERTY keyboard layout.
    PageUp = 85,
    /// The location of the page down  key on the QWERTY keyboard layout.
    PageDown = 86,

    /// The location of the right arrow  key on the QWERTY keyboard layout.
    ArrowRight = 89,
    // FIXME: Add Numpad
}
