use crate::keymap::{KeycapDescriptor, MappedKeyAction, NextKeyHint, PhysicalKey, build_ansi_keycaps};

const QWERTY_TABLE: &[(PhysicalKey, &str, &str, Option<&str>)] = &[
    (PhysicalKey::Backquote, "`", "~", None),
    (PhysicalKey::Digit1, "1", "!", None),
    (PhysicalKey::Digit2, "2", "@", None),
    (PhysicalKey::Digit3, "3", "#", None),
    (PhysicalKey::Digit4, "4", "$", None),
    (PhysicalKey::Digit5, "5", "%", None),
    (PhysicalKey::Digit6, "6", "^", None),
    (PhysicalKey::Digit7, "7", "&", None),
    (PhysicalKey::Digit8, "8", "*", None),
    (PhysicalKey::Digit9, "9", "(", None),
    (PhysicalKey::Digit0, "0", ")", None),
    (PhysicalKey::Minus, "-", "_", None),
    (PhysicalKey::Equal, "=", "+", None),
    (PhysicalKey::KeyQ, "q", "Q", None),
    (PhysicalKey::KeyW, "w", "W", None),
    (PhysicalKey::KeyE, "e", "E", None),
    (PhysicalKey::KeyR, "r", "R", None),
    (PhysicalKey::KeyT, "t", "T", None),
    (PhysicalKey::KeyY, "y", "Y", None),
    (PhysicalKey::KeyU, "u", "U", None),
    (PhysicalKey::KeyI, "i", "I", None),
    (PhysicalKey::KeyO, "o", "O", None),
    (PhysicalKey::KeyP, "p", "P", None),
    (PhysicalKey::BracketLeft, "[", "{", None),
    (PhysicalKey::BracketRight, "]", "}", None),
    (PhysicalKey::Backslash, "\\", "|", None),
    (PhysicalKey::KeyA, "a", "A", None),
    (PhysicalKey::KeyS, "s", "S", None),
    (PhysicalKey::KeyD, "d", "D", None),
    (PhysicalKey::KeyF, "f", "F", None),
    (PhysicalKey::KeyG, "g", "G", None),
    (PhysicalKey::KeyH, "h", "H", None),
    (PhysicalKey::KeyJ, "j", "J", None),
    (PhysicalKey::KeyK, "k", "K", None),
    (PhysicalKey::KeyL, "l", "L", None),
    (PhysicalKey::Semicolon, ";", ":", None),
    (PhysicalKey::Quote, "'", "\"", None),
    (PhysicalKey::KeyZ, "z", "Z", None),
    (PhysicalKey::KeyX, "x", "X", None),
    (PhysicalKey::KeyC, "c", "C", None),
    (PhysicalKey::KeyV, "v", "V", None),
    (PhysicalKey::KeyB, "b", "B", None),
    (PhysicalKey::KeyN, "n", "N", None),
    (PhysicalKey::KeyM, "m", "M", None),
    (PhysicalKey::Comma, ",", "<", None),
    (PhysicalKey::Period, ".", ">", None),
    (PhysicalKey::Slash, "/", "?", None),
];

/// Maps a physical key press under English QWERTY, respecting CapsLock XOR on letters.
pub fn map_key(key: PhysicalKey, shift: bool, caps_lock: bool) -> MappedKeyAction {
    if key == PhysicalKey::Space {
        return MappedKeyAction::FlushSpace;
    }
    let effective_shift = if key.is_alpha() {
        shift ^ caps_lock
    } else {
        shift
    };
    for &(entry_key, normal, shifted, _) in QWERTY_TABLE {
        if entry_key == key {
            let out = if effective_shift { shifted } else { normal };
            return MappedKeyAction::Insert(out.to_string());
        }
    }
    MappedKeyAction::NoOp
}

/// Computes the next physical key hint for an English grapheme.
pub fn hint_for_grapheme(target: &str, _typed_in_grapheme: &str) -> Option<NextKeyHint> {
    if target == " " {
        return Some(NextKeyHint {
            code: PhysicalKey::Space.as_dom_code().to_string(),
            shift: false,
            alt_gr: false,
            finger: PhysicalKey::Space.finger(),
            expected_label: "Space".to_string(),
        });
    }
    let ch = target.chars().next()?;
    let mut needle = [0u8; 4];
    let s = ch.encode_utf8(&mut needle);
    for &(key, normal, shifted, _) in QWERTY_TABLE {
        if normal == s {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: false,
                alt_gr: false,
                finger: key.finger(),
                expected_label: normal.to_string(),
            });
        }
        if shifted == s {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: true,
                alt_gr: false,
                finger: key.finger(),
                expected_label: shifted.to_string(),
            });
        }
    }
    None
}

/// Builds the 5-row ANSI virtual keyboard layout for English QWERTY.
pub fn keycaps() -> Vec<KeycapDescriptor> {
    build_ansi_keycaps(QWERTY_TABLE)
}
