use crate::keymap::{KeycapDescriptor, MappedKeyAction, NextKeyHint, PhysicalKey, build_ansi_keycaps};

const VIRAMA: char = '\u{094D}';
const SHORT_I: char = '\u{093F}';
const AA_MATRA: char = '\u{093E}';
const RA: char = '\u{0930}';

// Verified against SIL/Keyman remington_gail.kmn & Windows Hindi Indic Input 3 (CPCT).
// Columns: (PhysicalKey, normal, shift, alt_gr)
const REMINGTON_TABLE: &[(PhysicalKey, &str, &str, Option<&str>)] = &[
    (PhysicalKey::Backquote, "़", "द्य", Some("\u{200C}")),
    (PhysicalKey::Digit1, "1", "।", None),
    (PhysicalKey::Digit2, "2", "/", None),
    (PhysicalKey::Digit3, "3", "ः", None),
    (PhysicalKey::Digit4, "4", "*", None),
    (PhysicalKey::Digit5, "5", "-", None),
    (PhysicalKey::Digit6, "6", "‘", None),
    (PhysicalKey::Digit7, "7", "’", None),
    (PhysicalKey::Digit8, "8", "द्ध", None),
    (PhysicalKey::Digit9, "9", "त्र", None),
    (PhysicalKey::Digit0, "0", "ऋ", None),
    (PhysicalKey::Minus, ";", ".", Some("ञ")),
    (PhysicalKey::Equal, "ृ", "्", Some("+")),
    (PhysicalKey::KeyQ, "ु", "फ", Some("फ्")),
    (PhysicalKey::KeyW, "ू", "ॅ", Some("ॄ")),
    (PhysicalKey::KeyE, "म", "म्", Some("ें")),
    (PhysicalKey::KeyR, "त", "त्", Some("त्त")),
    (PhysicalKey::KeyT, "ज", "ज्", Some("ज्र")),
    (PhysicalKey::KeyY, "ल", "ल्", Some("द्द")),
    (PhysicalKey::KeyU, "न", "न्", Some("फ़")),
    (PhysicalKey::KeyI, "प", "प्", Some("फ")),
    (PhysicalKey::KeyO, "व", "व्", Some("व्न")),
    (PhysicalKey::KeyP, "च", "च्", Some("श")),
    (PhysicalKey::BracketLeft, "ख्", "क्ष्", Some("ख")),
    (PhysicalKey::BracketRight, ",", "द्व", Some("द्ध")),
    (PhysicalKey::Backslash, "(", ")", Some("ऽ")),
    (PhysicalKey::KeyA, "ं", "ा", Some("ह्ण")),
    (PhysicalKey::KeyS, "े", "ै", Some("भ")),
    (PhysicalKey::KeyD, "क", "क्", Some("ध")),
    (PhysicalKey::KeyF, "ि", "थ्", Some("य्")),
    (PhysicalKey::KeyG, "ह", "ळ", Some("घ")),
    (PhysicalKey::KeyH, "ी", "भ्", Some("ीं")),
    (PhysicalKey::KeyJ, "र", "श्र", Some("र्\u{200D}")),
    (PhysicalKey::KeyK, "ा", "ज्ञ", Some("्य")),
    (PhysicalKey::KeyL, "स", "स्", Some("स्त्र")),
    (PhysicalKey::Semicolon, "य", "रू", Some(";")),
    (PhysicalKey::Quote, "श्", "ष्", Some("श")),
    (PhysicalKey::KeyZ, "्र", "र्", Some("ो")),
    (PhysicalKey::KeyX, "ग", "ग्", Some("द्र")),
    (PhysicalKey::KeyC, "ब", "ब्", Some("ड़")),
    (PhysicalKey::KeyV, "अ", "ट", Some("ट्")),
    (PhysicalKey::KeyB, "इ", "ठ", Some("ई")),
    (PhysicalKey::KeyN, "द", "छ", Some("ण")),
    (PhysicalKey::KeyM, "उ", "ड", Some("ऊ")),
    (PhysicalKey::Comma, "ए", "ढ", Some("ँ")),
    (PhysicalKey::Period, "ण्", "झ", Some(".")),
    (PhysicalKey::Slash, "ध्", "घ्", Some("/")),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImeUndoSnapshot {
    pending_short_i: bool,
    emitted_text: String,
}

/// Stateful Remington GAIL IME implementing CPCT / Indic Input 3 typewriter behavior:
/// - Visual-order `ि` (`KeyF`) buffered before consonant clusters (`f + d` -> `कि`, `f + D + r` -> `क्ति`).
/// - Half-consonant completion via `k` (`ा`) stripping trailing virama (`F + k` -> `थ`).
/// - Compound vowel & matra ligatures (`v + k` -> `आ`, `k + s` -> `ो`, `b + Z` -> `ई`).
/// - Reph (`Shift+Z`) inserted before the current consonant syllable cluster.
/// - Safe `pending_short_i` flush when pressing `Space`.
/// - CapsLock is completely ignored (Shift/AltGr modifiers only).
#[derive(Debug, Default, Clone)]
pub struct RemingtonIme {
    pending_short_i: bool,
    history: Vec<ImeUndoSnapshot>,
}

impl RemingtonIme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pending_preview(&self) -> Option<&'static str> {
        if self.pending_short_i {
            Some("ि")
        } else {
            None
        }
    }

    /// Clears pending visual-order `ि` and word-level undo snapshots on word completion or reset.
    pub fn reset(&mut self) {
        self.pending_short_i = false;
        self.history.clear();
    }

    /// Processes a physical key under Remington GAIL.
    /// `current_word_buffer` is the text typed so far in the current active word/grapheme context.
    pub fn process_key(
        &mut self,
        key: PhysicalKey,
        shift: bool,
        alt_gr: bool,
        current_buffer: &str,
    ) -> (MappedKeyAction, String) {
        // Safeguard #3: Space safely flushes/clears any dangling `pending_short_i`.
        if key == PhysicalKey::Space {
            self.reset();
            return (MappedKeyAction::FlushSpace, " ".to_string());
        }

        // Visual-order `ि` (`KeyF` unshifted, no AltGr): buffer before consonant cluster.
        if key == PhysicalKey::KeyF && !shift && !alt_gr {
            self.history.push(ImeUndoSnapshot {
                pending_short_i: self.pending_short_i,
                emitted_text: current_buffer.to_string(),
            });
            self.pending_short_i = true;
            return (
                MappedKeyAction::PendingPreview("ि".to_string()),
                current_buffer.to_string(),
            );
        }

        let Some(raw) = lookup_raw(key, shift, alt_gr) else {
            return (MappedKeyAction::NoOp, current_buffer.to_string());
        };

        self.history.push(ImeUndoSnapshot {
            pending_short_i: self.pending_short_i,
            emitted_text: current_buffer.to_string(),
        });

        let mut next = current_buffer.to_string();

        // Reph (`Shift+Z`, no AltGr)
        if key == PhysicalKey::KeyZ && shift && !alt_gr {
            if next.ends_with('इ') {
                next.pop();
                next.push('ई');
            } else {
                insert_reph_before_syllable(&mut next);
            }
            self.pending_short_i = false;
            return (
                MappedKeyAction::ReplaceTail {
                    remove_chars: current_buffer.chars().count(),
                    insert: next.clone(),
                },
                next,
            );
        }

        // Half-consonant completion or `अ + ा -> आ` when pressing `k` (or `Shift+A` `ा`)
        if (key == PhysicalKey::KeyK && !shift && !alt_gr)
            || (key == PhysicalKey::KeyA && shift && !alt_gr)
        {
            if let Some(stripped) = strip_trailing_virama(&next) {
                next = stripped;
                // Keep `pending_short_i` waiting only if no full consonant ended yet?
                // Actually `F + k` -> `थ` IS now a full consonant, so attach `pending_short_i`!
                if self.pending_short_i {
                    next.push(SHORT_I);
                    self.pending_short_i = false;
                }
                return (
                    MappedKeyAction::ReplaceTail {
                        remove_chars: current_buffer.chars().count(),
                        insert: next.clone(),
                    },
                    next,
                );
            }
            if next.ends_with('अ') {
                next.pop();
                next.push('आ');
                self.pending_short_i = false;
                return (
                    MappedKeyAction::ReplaceTail {
                        remove_chars: current_buffer.chars().count(),
                        insert: next.clone(),
                    },
                    next,
                );
            }
        }

        // Compound vowel / matra rules (verified against remington_gail.kmn)
        if !alt_gr && apply_compound_rule(&mut next, key, shift) {
            return (
                MappedKeyAction::ReplaceTail {
                    remove_chars: current_buffer.chars().count(),
                    insert: next.clone(),
                },
                next,
            );
        }

        // Nukta folding (`Backquote` unshifted after nukta base consonant)
        if key == PhysicalKey::Backquote
            && !shift
            && !alt_gr
            && let Some(folded) = fold_nukta(&next)
        {
            next = folded;
            return (
                MappedKeyAction::ReplaceTail {
                    remove_chars: current_buffer.chars().count(),
                    insert: next.clone(),
                },
                next,
            );
        }

        next.push_str(raw);

        // Attach pending visual `ि` once a consonant cluster terminates on a full consonant
        // (i.e. ends with Devanagari consonant/conjunct that does NOT end in virama `्`).
        if self.pending_short_i && !raw.ends_with(VIRAMA) && ends_with_devanagari_consonant(&next) {
            next.push(SHORT_I);
            self.pending_short_i = false;
        }

        (
            MappedKeyAction::ReplaceTail {
                remove_chars: current_buffer.chars().count(),
                insert: next.clone(),
            },
            next,
        )
    }

    /// Reverts the most recent Remington keystroke step (for keystroke-level Backspace).
    /// Returns `Some((new_buffer, had_pending_before))` if an undo snapshot existed.
    pub fn undo_step(&mut self) -> Option<String> {
        self.history.pop().map(|snap| {
            self.pending_short_i = snap.pending_short_i;
            snap.emitted_text
        })
    }
}

fn lookup_raw(key: PhysicalKey, shift: bool, alt_gr: bool) -> Option<&'static str> {
    for &(entry_key, normal, shifted, alt) in REMINGTON_TABLE {
        if entry_key == key {
            if alt_gr && let Some(alt_val) = alt {
                return Some(alt_val);
            }
            return Some(if shift { shifted } else { normal });
        }
    }
    None
}

fn strip_trailing_virama(text: &str) -> Option<String> {
    let mut chars: Vec<char> = text.chars().collect();
    if chars.last() == Some(&VIRAMA) && chars.len() >= 2 {
        let prev = chars[chars.len() - 2];
        if is_devanagari_consonant(prev) {
            chars.pop();
            return Some(chars.into_iter().collect());
        }
    }
    None
}

fn apply_compound_rule(buf: &mut String, key: PhysicalKey, shift: bool) -> bool {
    let Some(last) = buf.chars().last() else {
        return false;
    };
    let replacement = match (last, key, shift) {
        ('आ', PhysicalKey::KeyS, false) => Some("ओ"),
        ('आ', PhysicalKey::KeyS, true) => Some("औ"),
        ('आ', PhysicalKey::KeyW, true) => Some("ऑ"),
        ('उ', PhysicalKey::KeyQ, false) => Some("ऊ"),
        ('ए', PhysicalKey::KeyS, false) => Some("ऐ"),
        ('ए', PhysicalKey::KeyW, true) => Some("ऍ"),
        (AA_MATRA, PhysicalKey::KeyS, false) => Some("ो"),
        (AA_MATRA, PhysicalKey::KeyS, true) => Some("ौ"),
        (AA_MATRA, PhysicalKey::KeyW, true) => Some("ॉ"),
        ('ृ', PhysicalKey::Equal, false) => Some("ॄ"),
        ('ॅ', PhysicalKey::KeyA, false) => Some("ँ"),
        ('।', PhysicalKey::Digit1, true) => Some("॥"),
        _ => None,
    };
    if let Some(rep) = replacement {
        buf.pop();
        buf.push_str(rep);
        true
    } else {
        false
    }
}

fn fold_nukta(text: &str) -> Option<String> {
    let last = text.chars().last()?;
    let folded = match last {
        'न' => '\u{0929}', // ऩ
        'र' => '\u{0931}', // ऱ
        'ळ' => '\u{0934}', // ऴ
        'क' => '\u{0958}', // क़
        'ख' => '\u{0959}', // ख़
        'ग' => '\u{095A}', // ग़
        'ज' => '\u{095B}', // ज़
        'ड' => '\u{095C}', // ड़
        'ढ' => '\u{095D}', // ढ़
        'फ' => '\u{095E}', // फ़
        'य' => '\u{095F}', // य़
        _ => return None,
    };
    let mut out = text.to_string();
    out.pop();
    out.push(folded);
    Some(out)
}

fn insert_reph_before_syllable(buf: &mut String) {
    let chars: Vec<char> = buf.chars().collect();
    if chars.is_empty() {
        buf.push(RA);
        buf.push(VIRAMA);
        return;
    }
    // Walk back past trailing matras/signs to the syllable's leading consonant cluster.
    let mut idx = chars.len();
    while idx > 0 && !is_devanagari_consonant(chars[idx - 1]) {
        idx -= 1;
    }
    if idx == 0 {
        buf.push(RA);
        buf.push(VIRAMA);
        return;
    }
    // Find start of consonant conjunct (Consonant + Virama)* + Consonant
    let mut cluster_start = idx - 1;
    while cluster_start >= 2 && chars[cluster_start - 1] == VIRAMA && is_devanagari_consonant(chars[cluster_start - 2]) {
        cluster_start -= 2;
    }
    let mut rebuilt = String::with_capacity(buf.len() + 6);
    for &c in &chars[..cluster_start] {
        rebuilt.push(c);
    }
    rebuilt.push(RA);
    rebuilt.push(VIRAMA);
    for &c in &chars[cluster_start..] {
        rebuilt.push(c);
    }
    *buf = rebuilt;
}

const fn is_devanagari_consonant(c: char) -> bool {
    matches!(c, '\u{0915}'..='\u{0939}' | '\u{0958}'..='\u{095F}')
}

fn ends_with_devanagari_consonant(text: &str) -> bool {
    text.chars().last().is_some_and(is_devanagari_consonant)
}

/// Computes the next expected physical key hint for a Remington GAIL target grapheme.
pub fn hint_for_grapheme(
    target: &str,
    typed_in_grapheme: &str,
    pending_ime: Option<&str>,
) -> Option<NextKeyHint> {
    if target == " " {
        return Some(NextKeyHint {
            code: PhysicalKey::Space.as_dom_code().to_string(),
            shift: false,
            alt_gr: false,
            finger: PhysicalKey::Space.finger(),
            expected_label: "Space".to_string(),
        });
    }
    // If the target ends with `ि` and user hasn't queued `pending_short_i` yet, guide `KeyF` first!
    if typed_in_grapheme.is_empty() && pending_ime != Some("ि") && target.ends_with(SHORT_I) {
        return Some(NextKeyHint {
            code: PhysicalKey::KeyF.as_dom_code().to_string(),
            shift: false,
            alt_gr: false,
            finger: PhysicalKey::KeyF.finger(),
            expected_label: "ि".to_string(),
        });
    }

    let remaining = target.strip_prefix(typed_in_grapheme).unwrap_or(target);
    // Match multi-char half-consonants/conjuncts (e.g., "क्", "क्ष्", "श्र", "ज्ञ") before single chars.
    for len in [3usize, 2, 1] {
        let candidate: String = remaining.chars().take(len).collect();
        if candidate.chars().count() < len {
            continue;
        }
        if let Some(hint) = find_remington_entry(&candidate) {
            return Some(hint);
        }
    }
    None
}

fn find_remington_entry(needle: &str) -> Option<NextKeyHint> {
    for &(key, normal, shifted, alt) in REMINGTON_TABLE {
        if normal == needle {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: false,
                alt_gr: false,
                finger: key.finger(),
                expected_label: normal.to_string(),
            });
        }
        if shifted == needle {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: true,
                alt_gr: false,
                finger: key.finger(),
                expected_label: shifted.to_string(),
            });
        }
        if let Some(a) = alt
            && a == needle
        {
            return Some(NextKeyHint {
                code: key.as_dom_code().to_string(),
                shift: false,
                alt_gr: true,
                finger: key.finger(),
                expected_label: a.to_string(),
            });
        }
    }
    None
}

/// Builds the 5-row ANSI virtual keyboard layout for Hindi Remington GAIL.
pub fn keycaps() -> Vec<KeycapDescriptor> {
    build_ansi_keycaps(REMINGTON_TABLE)
}
