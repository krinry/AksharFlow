use crate::keymap::{KeycapDescriptor, MappedKeyAction, NextKeyHint, PhysicalKey, build_ansi_keycaps};

// Standard C-DAC Hindi InScript + InScript2 AltGr extensions (verified against hi-inscript2.mim).
// Columns: (PhysicalKey, normal, shift, alt_gr)
const INSCRIPT_TABLE: &[(PhysicalKey, &str, &str, Option<&str>)] = &[
    (PhysicalKey::Backquote, "ॊ", "ऒ", None),
    (PhysicalKey::Digit1, "१", "ऍ", Some("\u{200D}")),
    (PhysicalKey::Digit2, "२", "ॅ", Some("\u{200C}")),
    (PhysicalKey::Digit3, "३", "्र", None),
    (PhysicalKey::Digit4, "४", "र्", Some("₹")),
    (PhysicalKey::Digit5, "५", "ज्ञ", None),
    (PhysicalKey::Digit6, "६", "त्र", None),
    (PhysicalKey::Digit7, "७", "क्ष", None),
    (PhysicalKey::Digit8, "८", "श्र", None),
    (PhysicalKey::Digit9, "९", "(", None),
    (PhysicalKey::Digit0, "०", ")", None),
    (PhysicalKey::Minus, "-", "ः", None),
    (PhysicalKey::Equal, "ृ", "ऋ", Some("ॄ")),
    (PhysicalKey::KeyQ, "ौ", "औ", None),
    (PhysicalKey::KeyW, "ै", "ऐ", None),
    (PhysicalKey::KeyE, "ा", "आ", None),
    (PhysicalKey::KeyR, "ी", "ई", Some("ॣ")),
    (PhysicalKey::KeyT, "ू", "ऊ", None),
    (PhysicalKey::KeyY, "ब", "भ", None),
    (PhysicalKey::KeyU, "ह", "ङ", None),
    (PhysicalKey::KeyI, "ग", "घ", Some("ग़")),
    (PhysicalKey::KeyO, "द", "ध", None),
    (PhysicalKey::KeyP, "ज", "झ", Some("ज़")),
    (PhysicalKey::BracketLeft, "ड", "ढ", Some("ड़")),
    (PhysicalKey::BracketRight, "़", "ञ", None),
    (PhysicalKey::Backslash, "ॉ", "ऑ", None),
    (PhysicalKey::KeyA, "ो", "ओ", None),
    (PhysicalKey::KeyS, "े", "ए", None),
    (PhysicalKey::KeyD, "्", "अ", None),
    (PhysicalKey::KeyF, "ि", "इ", Some("ॢ")),
    (PhysicalKey::KeyG, "ु", "उ", None),
    (PhysicalKey::KeyH, "प", "फ", Some("फ़")),
    (PhysicalKey::KeyJ, "र", "ऱ", None),
    (PhysicalKey::KeyK, "क", "ख", Some("क़")),
    (PhysicalKey::KeyL, "त", "थ", None),
    (PhysicalKey::Semicolon, "च", "छ", None),
    (PhysicalKey::Quote, "ट", "ठ", None),
    (PhysicalKey::KeyZ, "ॆ", "ऎ", None),
    (PhysicalKey::KeyX, "ं", "ँ", Some("ॐ")),
    (PhysicalKey::KeyC, "म", "ण", None),
    (PhysicalKey::KeyV, "न", "ऩ", None),
    (PhysicalKey::KeyB, "व", "ऴ", None),
    (PhysicalKey::KeyN, "ल", "ळ", None),
    (PhysicalKey::KeyM, "स", "श", None),
    (PhysicalKey::Comma, ",", "ष", Some("॰")),
    (PhysicalKey::Period, ".", "।", Some("॥")),
    (PhysicalKey::Slash, "य", "य़", None),
];

/// Maps a physical key press under Hindi InScript.
/// CapsLock is intentionally ignored so accidental CapsLock never triggers Shift-layer letters.
pub fn map_key(key: PhysicalKey, shift: bool, alt_gr: bool) -> MappedKeyAction {
    if key == PhysicalKey::Space {
        return MappedKeyAction::FlushSpace;
    }
    for &(entry_key, normal, shifted, alt) in INSCRIPT_TABLE {
        if entry_key == key {
            if alt_gr
                && let Some(alt_val) = alt
            {
                return MappedKeyAction::Insert(alt_val.to_string());
            }
            let out = if shift { shifted } else { normal };
            return MappedKeyAction::Insert(out.to_string());
        }
    }
    MappedKeyAction::NoOp
}

/// Resolves the next physical key hint for an InScript target grapheme given typed progress.
pub fn hint_for_grapheme(target: &str, typed_in_grapheme: &str) -> Option<NextKeyHint> {
    if target == " " {
        return Some(NextKeyHint {
            code: PhysicalKey::Space.as_dom_code().to_string(),
            shift: false,
            alt_gr: false,
            finger: PhysicalKey::Space.finger(),
            expected_label: "Space".to_string(),
        });
    }
    let remaining = target.strip_prefix(typed_in_grapheme)?;
    if remaining.is_empty() {
        return None;
    }
    // Prefer whole-conjunct shortcut only when starting from empty partial buffer (e.g. क्ष, त्र, ज्ञ, श्र)
    if typed_in_grapheme.is_empty()
        && let Some(hint) = find_exact_sequence(remaining)
    {
        return Some(hint);
    }
    let next_char = remaining.chars().next()?;
    let mut buf = [0u8; 4];
    let s = next_char.encode_utf8(&mut buf);
    find_exact_sequence(s)
}

fn find_exact_sequence(seq: &str) -> Option<NextKeyHint> {
    for &(key, normal, shifted, alt) in INSCRIPT_TABLE {
        if normal == seq {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: false,
                alt_gr: false,
                finger: key.finger(),
                expected_label: normal.to_string(),
            });
        }
        if shifted == seq {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: true,
                alt_gr: false,
                finger: key.finger(),
                expected_label: shifted.to_string(),
            });
        }
        if let Some(alt_val) = alt
            && alt_val == seq
        {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: false,
                alt_gr: true,
                finger: key.finger(),
                expected_label: alt_val.to_string(),
            });
        }
    }
    None
}

/// Builds the 5-row ANSI virtual keyboard layout for Hindi InScript.
pub fn keycaps() -> Vec<KeycapDescriptor> {
    build_ansi_keycaps(INSCRIPT_TABLE)
}
