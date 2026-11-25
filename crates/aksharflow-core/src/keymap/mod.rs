pub mod inscript;
pub mod qwerty;
pub mod remington_gail;

use serde::{Deserialize, Serialize};

/// Supported keyboard layouts built into AksharFlow (zero OS layout dependency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum LayoutKind {
    EnQwerty,
    HiInscript,
    HiRemingtonGail,
}

impl std::fmt::Display for LayoutKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnQwerty => write!(f, "en_qwerty"),
            Self::HiInscript => write!(f, "hi_inscript"),
            Self::HiRemingtonGail => write!(f, "hi_remington_gail"),
        }
    }
}

/// Touch-typing finger assignment for the Virtual Keyboard & Finger Guide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum FingerId {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    Thumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
}

/// Canonical physical ANSI key identifiers matching `KeyboardEvent.code`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub enum PhysicalKey {
    Backquote,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Minus,
    Equal,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    BracketLeft,
    BracketRight,
    Backslash,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,
    Space,
    Backspace,
}

impl PhysicalKey {
    /// Parses a DOM `KeyboardEvent.code` string into a canonical `PhysicalKey`.
    pub fn from_dom_code(code: &str) -> Option<Self> {
        match code {
            "Backquote" => Some(Self::Backquote),
            "Digit1" => Some(Self::Digit1),
            "Digit2" => Some(Self::Digit2),
            "Digit3" => Some(Self::Digit3),
            "Digit4" => Some(Self::Digit4),
            "Digit5" => Some(Self::Digit5),
            "Digit6" => Some(Self::Digit6),
            "Digit7" => Some(Self::Digit7),
            "Digit8" => Some(Self::Digit8),
            "Digit9" => Some(Self::Digit9),
            "Digit0" => Some(Self::Digit0),
            "Minus" => Some(Self::Minus),
            "Equal" => Some(Self::Equal),
            "KeyQ" => Some(Self::KeyQ),
            "KeyW" => Some(Self::KeyW),
            "KeyE" => Some(Self::KeyE),
            "KeyR" => Some(Self::KeyR),
            "KeyT" => Some(Self::KeyT),
            "KeyY" => Some(Self::KeyY),
            "KeyU" => Some(Self::KeyU),
            "KeyI" => Some(Self::KeyI),
            "KeyO" => Some(Self::KeyO),
            "KeyP" => Some(Self::KeyP),
            "BracketLeft" => Some(Self::BracketLeft),
            "BracketRight" => Some(Self::BracketRight),
            "Backslash" => Some(Self::Backslash),
            "KeyA" => Some(Self::KeyA),
            "KeyS" => Some(Self::KeyS),
            "KeyD" => Some(Self::KeyD),
            "KeyF" => Some(Self::KeyF),
            "KeyG" => Some(Self::KeyG),
            "KeyH" => Some(Self::KeyH),
            "KeyJ" => Some(Self::KeyJ),
            "KeyK" => Some(Self::KeyK),
            "KeyL" => Some(Self::KeyL),
            "Semicolon" => Some(Self::Semicolon),
            "Quote" => Some(Self::Quote),
            "KeyZ" => Some(Self::KeyZ),
            "KeyX" => Some(Self::KeyX),
            "KeyC" => Some(Self::KeyC),
            "KeyV" => Some(Self::KeyV),
            "KeyB" => Some(Self::KeyB),
            "KeyN" => Some(Self::KeyN),
            "KeyM" => Some(Self::KeyM),
            "Comma" => Some(Self::Comma),
            "Period" => Some(Self::Period),
            "Slash" => Some(Self::Slash),
            "Space" => Some(Self::Space),
            "Backspace" => Some(Self::Backspace),
            _ => None,
        }
    }

    /// Returns the canonical DOM `KeyboardEvent.code` string.
    pub const fn as_dom_code(self) -> &'static str {
        match self {
            Self::Backquote => "Backquote",
            Self::Digit1 => "Digit1",
            Self::Digit2 => "Digit2",
            Self::Digit3 => "Digit3",
            Self::Digit4 => "Digit4",
            Self::Digit5 => "Digit5",
            Self::Digit6 => "Digit6",
            Self::Digit7 => "Digit7",
            Self::Digit8 => "Digit8",
            Self::Digit9 => "Digit9",
            Self::Digit0 => "Digit0",
            Self::Minus => "Minus",
            Self::Equal => "Equal",
            Self::KeyQ => "KeyQ",
            Self::KeyW => "KeyW",
            Self::KeyE => "KeyE",
            Self::KeyR => "KeyR",
            Self::KeyT => "KeyT",
            Self::KeyY => "KeyY",
            Self::KeyU => "KeyU",
            Self::KeyI => "KeyI",
            Self::KeyO => "KeyO",
            Self::KeyP => "KeyP",
            Self::BracketLeft => "BracketLeft",
            Self::BracketRight => "BracketRight",
            Self::Backslash => "Backslash",
            Self::KeyA => "KeyA",
            Self::KeyS => "KeyS",
            Self::KeyD => "KeyD",
            Self::KeyF => "KeyF",
            Self::KeyG => "KeyG",
            Self::KeyH => "KeyH",
            Self::KeyJ => "KeyJ",
            Self::KeyK => "KeyK",
            Self::KeyL => "KeyL",
            Self::Semicolon => "Semicolon",
            Self::Quote => "Quote",
            Self::KeyZ => "KeyZ",
            Self::KeyX => "KeyX",
            Self::KeyC => "KeyC",
            Self::KeyV => "KeyV",
            Self::KeyB => "KeyB",
            Self::KeyN => "KeyN",
            Self::KeyM => "KeyM",
            Self::Comma => "Comma",
            Self::Period => "Period",
            Self::Slash => "Slash",
            Self::Space => "Space",
            Self::Backspace => "Backspace",
        }
    }

    /// Returns true if this physical key corresponds to an English alphabetic letter.
    pub const fn is_alpha(self) -> bool {
        matches!(
            self,
            Self::KeyQ
                | Self::KeyW
                | Self::KeyE
                | Self::KeyR
                | Self::KeyT
                | Self::KeyY
                | Self::KeyU
                | Self::KeyI
                | Self::KeyO
                | Self::KeyP
                | Self::KeyA
                | Self::KeyS
                | Self::KeyD
                | Self::KeyF
                | Self::KeyG
                | Self::KeyH
                | Self::KeyJ
                | Self::KeyK
                | Self::KeyL
                | Self::KeyZ
                | Self::KeyX
                | Self::KeyC
                | Self::KeyV
                | Self::KeyB
                | Self::KeyN
                | Self::KeyM
        )
    }

    /// Standard ANSI touch-typing finger zone for each physical key.
    pub const fn finger(self) -> FingerId {
        match self {
            Self::Backquote
            | Self::Digit1
            | Self::KeyQ
            | Self::KeyA
            | Self::KeyZ
            | Self::Digit0
            | Self::Minus
            | Self::Equal
            | Self::KeyP
            | Self::BracketLeft
            | Self::BracketRight
            | Self::Backslash
            | Self::Semicolon
            | Self::Quote
            | Self::Slash
            | Self::Backspace => {
                if matches!(
                    self,
                    Self::Backquote | Self::Digit1 | Self::KeyQ | Self::KeyA | Self::KeyZ
                ) {
                    FingerId::LeftPinky
                } else {
                    FingerId::RightPinky
                }
            }
            Self::Digit2 | Self::KeyW | Self::KeyS | Self::KeyX => FingerId::LeftRing,
            Self::Digit3 | Self::KeyE | Self::KeyD | Self::KeyC => FingerId::LeftMiddle,
            Self::Digit4
            | Self::Digit5
            | Self::KeyR
            | Self::KeyT
            | Self::KeyF
            | Self::KeyG
            | Self::KeyV
            | Self::KeyB => FingerId::LeftIndex,
            Self::Digit6
            | Self::Digit7
            | Self::KeyY
            | Self::KeyU
            | Self::KeyH
            | Self::KeyJ
            | Self::KeyN
            | Self::KeyM => FingerId::RightIndex,
            Self::Digit8 | Self::KeyI | Self::KeyK | Self::Comma => FingerId::RightMiddle,
            Self::Digit9 | Self::KeyO | Self::KeyL | Self::Period => FingerId::RightRing,
            Self::Space => FingerId::Thumb,
        }
    }
}

/// Incoming hardware or touch-keyboard key event sent by the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct KeyInputEvent {
    pub code: String,
    pub shift: bool,
    pub alt_gr: bool,
    pub ctrl: bool,
    pub caps_lock: bool,
    pub timestamp_ms: f64,
}

/// Hint telling the Virtual Keyboard and Finger Guide what physical key to press next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct NextKeyHint {
    pub code: String,
    pub shift: bool,
    pub alt_gr: bool,
    pub finger: FingerId,
    pub expected_label: String,
}

/// Metadata for a single keycap on the interactive Virtual Keyboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct KeycapDescriptor {
    pub code: String,
    pub row: u8,
    pub normal_label: String,
    pub shift_label: String,
    pub alt_gr_label: Option<String>,
    pub finger: FingerId,
    pub width_u: f32,
}

/// Unified keymapper trait implemented by each keyboard layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedKeyAction {
    Insert(String),
    ReplaceTail { remove_chars: usize, insert: String },
    PendingPreview(String),
    FlushSpace,
    NoOp,
}

/// Returns the interactive keycap descriptors for the requested layout.
pub fn layout_keycaps(layout: LayoutKind) -> Vec<KeycapDescriptor> {
    match layout {
        LayoutKind::EnQwerty => qwerty::keycaps(),
        LayoutKind::HiInscript => inscript::keycaps(),
        LayoutKind::HiRemingtonGail => remington_gail::keycaps(),
    }
}

/// Shared key-row metadata helper used across all layouts to build `KeycapDescriptor` entries.
pub(crate) fn build_ansi_keycaps(
    table: &[(PhysicalKey, &str, &str, Option<&str>)],
) -> Vec<KeycapDescriptor> {
    const ROWS: &[&[(PhysicalKey, f32)]] = &[
        &[
            (PhysicalKey::Backquote, 1.0),
            (PhysicalKey::Digit1, 1.0),
            (PhysicalKey::Digit2, 1.0),
            (PhysicalKey::Digit3, 1.0),
            (PhysicalKey::Digit4, 1.0),
            (PhysicalKey::Digit5, 1.0),
            (PhysicalKey::Digit6, 1.0),
            (PhysicalKey::Digit7, 1.0),
            (PhysicalKey::Digit8, 1.0),
            (PhysicalKey::Digit9, 1.0),
            (PhysicalKey::Digit0, 1.0),
            (PhysicalKey::Minus, 1.0),
            (PhysicalKey::Equal, 1.0),
            (PhysicalKey::Backspace, 2.0),
        ],
        &[
            (PhysicalKey::KeyQ, 1.0),
            (PhysicalKey::KeyW, 1.0),
            (PhysicalKey::KeyE, 1.0),
            (PhysicalKey::KeyR, 1.0),
            (PhysicalKey::KeyT, 1.0),
            (PhysicalKey::KeyY, 1.0),
            (PhysicalKey::KeyU, 1.0),
            (PhysicalKey::KeyI, 1.0),
            (PhysicalKey::KeyO, 1.0),
            (PhysicalKey::KeyP, 1.0),
            (PhysicalKey::BracketLeft, 1.0),
            (PhysicalKey::BracketRight, 1.0),
            (PhysicalKey::Backslash, 1.5),
        ],
        &[
            (PhysicalKey::KeyA, 1.0),
            (PhysicalKey::KeyS, 1.0),
            (PhysicalKey::KeyD, 1.0),
            (PhysicalKey::KeyF, 1.0),
            (PhysicalKey::KeyG, 1.0),
            (PhysicalKey::KeyH, 1.0),
            (PhysicalKey::KeyJ, 1.0),
            (PhysicalKey::KeyK, 1.0),
            (PhysicalKey::KeyL, 1.0),
            (PhysicalKey::Semicolon, 1.0),
            (PhysicalKey::Quote, 1.0),
        ],
        &[
            (PhysicalKey::KeyZ, 1.0),
            (PhysicalKey::KeyX, 1.0),
            (PhysicalKey::KeyC, 1.0),
            (PhysicalKey::KeyV, 1.0),
            (PhysicalKey::KeyB, 1.0),
            (PhysicalKey::KeyN, 1.0),
            (PhysicalKey::KeyM, 1.0),
            (PhysicalKey::Comma, 1.0),
            (PhysicalKey::Period, 1.0),
            (PhysicalKey::Slash, 1.0),
        ],
        &[(PhysicalKey::Space, 6.25)],
    ];

    let mut out = Vec::with_capacity(50);
    for (row_idx, row) in ROWS.iter().enumerate() {
        for &(key, width_u) in *row {
            let (normal, shift, alt_gr) = match key {
                PhysicalKey::Backspace => ("⌫", "⌫", None),
                PhysicalKey::Space => ("Space", "Space", None),
                _ => table
                    .iter()
                    .find(|&&(k, _, _, _)| k == key)
                    .map(|&(_, n, s, a)| (n, s, a))
                    .unwrap_or(("", "", None)),
            };
            out.push(KeycapDescriptor {
                code: key.as_dom_code().to_string(),
                row: row_idx as u8,
                normal_label: normal.to_string(),
                shift_label: shift.to_string(),
                alt_gr_label: alt_gr.map(str::to_string),
                finger: key.finger(),
                width_u,
            });
        }
    }
    out
}

/// Computes the next expected physical key hint for a single target grapheme.
pub fn hint_for_grapheme(
    layout: LayoutKind,
    target_grapheme: &str,
    typed_in_grapheme: &str,
    pending_ime: Option<&str>,
) -> Option<NextKeyHint> {
    match layout {
        LayoutKind::EnQwerty => qwerty::hint_for_grapheme(target_grapheme, typed_in_grapheme),
        LayoutKind::HiInscript => inscript::hint_for_grapheme(target_grapheme, typed_in_grapheme),
        LayoutKind::HiRemingtonGail => {
            remington_gail::hint_for_grapheme(target_grapheme, typed_in_grapheme, pending_ime)
        }
    }
}
