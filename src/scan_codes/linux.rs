//! Helper enum to define scan codes on the QWERTY keyboard layout.

/// The key locations as defined by the keys on the QWERTY keyboard layout.
///
/// The [`u32`] representation of this enum are scan codes of the corresponding keys on X-like Linux systems.
/// See <https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h>.
#[repr(u32)]
pub enum QwertyScanCode {
    /// The location of the Escape/Esc key on the QWERTY keyboard layout.
    Escape = 1,
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
    /// The location of the back(space) key on the QWERTY keyboard layout.
    Backspace = 14,
    /// The location of the tabulator key on the QWERTY keyboard layout.
    Tab = 15,
    /// The location of the `Q`  key on the QWERTY keyboard layout.
    Q = 16,
    /// The location of the `W`  key on the QWERTY keyboard layout.
    W = 17,
    /// The location of the `E`  key on the QWERTY keyboard layout.
    E = 18,
    /// The location of the `R`  key on the QWERTY keyboard layout.
    R = 19,
    /// The location of the `T`  key on the QWERTY keyboard layout.
    T = 20,
    /// The location of the `Y`  key on the QWERTY keyboard layout.
    Y = 21,
    /// The location of the `U`  key on the QWERTY keyboard layout.
    U = 22,
    /// The location of the `I`  key on the QWERTY keyboard layout.
    I = 23,
    /// The location of the `O`  key on the QWERTY keyboard layout.
    O = 24,
    /// The location of the `P`  key on the QWERTY keyboard layout.
    P = 25,
    /// The location of the `[`  key on the QWERTY keyboard layout.
    BracketLeft = 26,
    /// The location of the `]`  key on the QWERTY keyboard layout.
    BracketRight = 27,
    /// The location of the Enter/Return key on the QWERTY keyboard layout.
    Enter = 28,
    /// The location of the left Control key on the QWERTY keyboard layout.
    ControlLeft = 29,
    /// The location of the `A`  key on the QWERTY keyboard layout.
    A = 30,
    /// The location of the `S`  key on the QWERTY keyboard layout.
    S = 31,
    /// The location of the `D`  key on the QWERTY keyboard layout.
    D = 32,
    /// The location of the `F`  key on the QWERTY keyboard layout.
    F = 33,
    /// The location of the `G`  key on the QWERTY keyboard layout.
    G = 34,
    /// The location of the `H`  key on the QWERTY keyboard layout.
    H = 35,
    /// The location of the `J`  key on the QWERTY keyboard layout.
    J = 36,
    /// The location of the `K`  key on the QWERTY keyboard layout.
    K = 37,
    /// The location of the `L`  key on the QWERTY keyboard layout.
    L = 38,
    /// The location of the `;`  key on the QWERTY keyboard layout.
    SemiColon = 39,
    /// The location of the `'`  key on the QWERTY keyboard layout.
    Apostrophe = 40,
    /// The location of the `` ` `` key on the QWERTY keyboard layout.
    Backtick = 41,
    /// The location of the left Shift key on the QWERTY keyboard layout.
    ShiftLeft = 42,
    /// The location of the `\` on the QWERTY keyboard layout.
    Backslash = 43,
    /// The location of the `Z` key on the QWERTY keyboard layout.
    Z = 44,
    /// The location of the `X` key on the QWERTY keyboard layout.
    X = 45,
    /// The location of the `C` key on the QWERTY keyboard layout.
    C = 46,
    /// The location of the `V key on the QWERTY keyboard layout.
    V = 47,
    /// The location of the `B` key on the QWERTY keyboard layout.
    B = 48,
    /// The location of the `N` key on the QWERTY keyboard layout.
    N = 49,
    /// The location of the `M` key on the QWERTY keyboard layout.
    M = 50,
    /// The location of the `,` key on the QWERTY keyboard layout.
    Comma = 51,
    /// The location of the `.` key on the QWERTY keyboard layout.
    Period = 52,
    /// The location of the `/` key on the QWERTY keyboard layout.
    Slash = 53,
    /// The location of the right Shift key on the QWERTY keyboard layout.
    ShiftRight = 54,
    /// The location of the `*` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadDivide` on Apple keyboards.
    NumpadMultiply = 55,
    /// The location of the left Alt  key on the QWERTY keyboard layout.
    /// Maps to left Option key on Apple keyboards.
    AltLeft = 56,
    /// The location of the Space key on the QWERTY keyboard layout.
    Space = 57,
    /// The location of the caps lock  key on the QWERTY keyboard layout.
    CapsLock = 58,
    /// The location of the `F1` key on the QWERTY keyboard layout.
    F1 = 59,
    /// The location of the `F2` key on the QWERTY keyboard layout.
    F2 = 60,
    /// The location of the `F3` key on the QWERTY keyboard layout.
    F3 = 61,
    /// The location of the `F4` key on the QWERTY keyboard layout.
    F4 = 62,
    /// The location of the `F5` key on the QWERTY keyboard layout.
    F5 = 63,
    /// The location of the `F6` key on the QWERTY keyboard layout.
    F6 = 64,
    /// The location of the `F7` key on the QWERTY keyboard layout.
    F7 = 65,
    /// The location of the `F8` key on the QWERTY keyboard layout.
    F8 = 66,
    /// The location of the `F9` key on the QWERTY keyboard layout.
    F9 = 67,
    /// The location of the `F10` key on the QWERTY keyboard layout.
    F10 = 68,
    /// The location of the Numlock key on the QWERTY keyboard layout.
    /// Maps to `NumpadClear` on Apple keyboards.
    Numlock = 69,
    /// The location of the Scroll / Scroll Lock key on the QWERTY keyboard layout.
    /// Maps to the `F14` key on Apple keyboards.
    Scroll = 70,
    /// The location of the `7` key on the numpad of the QWERTY keyboard layout.
    Numpad7 = 71,
    /// The location of the `8` key on the numpad of the QWERTY keyboard layout.
    Numpad8 = 72,
    /// The location of the `9` key on the numpad of the QWERTY keyboard layout.
    Numpad9 = 73,
    /// The location of the `*` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadMultiply` on Apple keyboards.
    NumpadSubtract = 74,
    /// The location of the `4` key on the numpad of the QWERTY keyboard layout.
    Numpad4 = 75,
    /// The location of the `5` key on the numpad of the QWERTY keyboard layout.
    Numpad5 = 76,
    /// The location of the `6` key on the numpad of the QWERTY keyboard layout.
    Numpad6 = 77,
    /// The location of the `+` key on the numpad of the QWERTY keyboard layout.
    NumpadAdd = 78,
    /// The location of the `1` key on the numpad of the QWERTY keyboard layout.
    Numpad1 = 79,
    /// The location of the `2` key on the numpad of the QWERTY keyboard layout.
    Numpad2 = 80,
    /// The location of the `3` key on the numpad of the QWERTY keyboard layout.
    Numpad3 = 81,
    /// The location of the `0` key on the numpad of the QWERTY keyboard layout.
    Numpad0 = 82,
    /// The location of the `.` key on the numpad of the QWERTY keyboard layout.
    NumpadDecimal = 83,
    // KEY_ZENKAKUHANKAKU = 85,
    // KEY_102ND = 86,
    /// The location of the `F11` key on the QWERTY keyboard layout.
    F11 = 87,
    /// The location of the `F12` key on the QWERTY keyboard layout.
    F12 = 88,
    // KEY_R0 = 89,
    // KEY_KATAKANA = 90,
    // KEY_HIRAGANA = 91,
    // KEY_HENKAN = 92,
    // KEY_KATAKANAHIRAGANA = 93,
    // KEY_MUHENKAN = 94,
    // KEY_KPJPCOMMA = 95,
    /// The location of the Enter key on the numpad of the QWERTY keyboard layout.
    NumpadEnter = 96,
    /// The location of the right Control key on the QWERTY keyboard layout.
    ControlRight = 97,
    /// The location of the `/` key on the numpad of the QWERTY keyboard layout.
    /// Maps to `NumpadEquals` on Apple keyboards.
    NumpadDivide = 98,
    /// The location of the Alt+Sysrq key on the QWERTY keyboard layout.
    AltSysrq = 99,
    /// The location of the right Alt key on the QWERTY keyboard layout.
    /// Maps to right Option key on Apple keyboards.
    AltRight = 100,
    // KEY_LINEFEED = 101,
    /// The location of the Home key on the QWERTY keyboard layout.
    Home = 102,
    /// The location of the Arrow Up key on the QWERTY keyboard layout.
    Up = 103,
    /// The location of the Page Up key on the QWERTY keyboard layout.
    PageUp = 104,
    /// The location of the Arrow Left key on the QWERTY keyboard layout.
    Left = 105,
    /// The location of the Arrow Right key on the QWERTY keyboard layout.
    Right = 106,
    /// The location of the End key on the QWERTY keyboard layout.
    End = 107,
    /// The location of the Arrow Down key on the QWERTY keyboard layout.
    Down = 108,
    /// The location of the Page Down key on the QWERTY keyboard layout.
    PageDown = 109,
    /// The location of the Insert key on the QWERTY keyboard layout.
    /// Maps to the Help key on Apple keyboards.
    Insert = 110,
    /// The location of the Delete key on the QWERTY keyboard layout.
    Delete = 111,
    // KEY_MACRO = 112,
    // KEY_MUTE = 113,
    // KEY_VOLUMEDOWN = 114,
    // KEY_VOLUMEDOWN = 115,
    /// The location of the Power key on the QWERTY keyboard layout.
    Power = 116,
    // KEY_KPEQUAL = 117,
    // KEY_KPPLUSMINUS = 118,
    /// The location of the Pause key on the QWERTY keyboard layout.
    /// Maps to the `F15` key on Apple keyboards.
    Pause = 119,
    // KEY_SCALE = 120,
    // KEY_KPCOMMA = 121,
    // KEY_HANGEUL = 122,
    // KEY_HANJA = 123,
    // KEY_YEN = 124,
    /// The location of the left Windows key on the QWERTY keyboard layout.
    /// Maps to the Command key on Apple keyboards.
    SuperLeft = 125,
    /// The location of the right Windows key on the QWERTY keyboard layout.
    SuperRight = 126,
    // KEY_COMPOSE = 127,
    // KEY_STOP = 128,
    // KEY_AGAIN = 129,
    // KEY_PROPS = 130,
    // KEY_UNDO = 131,
    // KEY_FRONT = 132,
    // KEY_COPY = 133,
    // KEY_OPEN = 134,
    // KEY_PASTE = 135,
    // KEY_FIND = 136,
    // KEY_CUT = 137,
    // KEY_HELP = 138,
    /// The location of the Menu key on the QWERTY keyboard layout.
    Menu = 139,
    // KEY_CALC = 140,
    // KEY_SETUP = 141,
    /// The location of the Sleep key on the QWERTY keyboard layout.
    Sleep = 142,
    /// The location of the Wake key on the QWERTY keyboard layout.
    Wake = 143,
    // Keys to figure out later:
    // /// A key not available on the US QWERTY layout.
    // ///
    // /// This is for example the `#` key on other layouts.
    // NonUs1 = 0x00,
    // /// The location of the Snapshot / Print Screen key on the QWERTY keyboard layout.
    // /// Maps to the `F13` key on Apple keyboards.
    // Snapshot = 0xe0_37,
    // /// The location of the Ctrl+Break key on the QWERTY keyboard layout.
    // CtrlBreak = 0xe0_46,
}
