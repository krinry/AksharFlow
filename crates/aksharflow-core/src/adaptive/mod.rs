use std::collections::{HashMap, HashSet};

use chrono::Utc;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::corpus::Corpus;
use crate::error::AksharError;
use crate::grapheme::normalize_nfc;
use crate::keymap::{FingerId, LayoutKind, PhysicalKey};
use crate::storage::{AppSettings, KeyStatRecord};

pub const MAX_LATENCY_CAP_MS: f64 = 2500.0;
pub const MIN_SAMPLES_FOR_UNLOCK: u32 = 12;
const EMA_ALPHA: f64 = 0.25;

/// Static definition of a progressive key unlock step for a layout.
#[derive(Debug, Clone, Copy)]
pub struct UnlockSpec {
    pub key_id: &'static str,
    pub physical_key: PhysicalKey,
    pub shift: bool,
    pub alt_gr: bool,
    pub initial_unlocked: bool,
}

/// Toast/UI notification payload emitted when a learner unlocks a new key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct UnlockedKeyEvent {
    pub layout: LayoutKind,
    pub key_id: String,
    pub physical_code: String,
    pub requires_shift: bool,
    pub requires_alt_gr: bool,
    pub finger: FingerId,
    pub unlock_order: u32,
    pub total_keys: u32,
}

const EN_QWERTY_SEQUENCE: &[UnlockSpec] = &[
    // Home Row (7 letters initially unlocked)
    UnlockSpec { key_id: "a", physical_key: PhysicalKey::KeyA, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "s", physical_key: PhysicalKey::KeyS, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "d", physical_key: PhysicalKey::KeyD, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "f", physical_key: PhysicalKey::KeyF, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "j", physical_key: PhysicalKey::KeyJ, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "k", physical_key: PhysicalKey::KeyK, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "l", physical_key: PhysicalKey::KeyL, shift: false, alt_gr: false, initial_unlocked: true },
    // Progressive tiers ordered by ergonomic comfort & English frequency
    UnlockSpec { key_id: "e", physical_key: PhysicalKey::KeyE, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "n", physical_key: PhysicalKey::KeyN, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "i", physical_key: PhysicalKey::KeyI, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "r", physical_key: PhysicalKey::KeyR, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "t", physical_key: PhysicalKey::KeyT, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "o", physical_key: PhysicalKey::KeyO, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "g", physical_key: PhysicalKey::KeyG, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "h", physical_key: PhysicalKey::KeyH, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "u", physical_key: PhysicalKey::KeyU, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "c", physical_key: PhysicalKey::KeyC, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "m", physical_key: PhysicalKey::KeyM, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "p", physical_key: PhysicalKey::KeyP, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "w", physical_key: PhysicalKey::KeyW, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "y", physical_key: PhysicalKey::KeyY, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "b", physical_key: PhysicalKey::KeyB, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "v", physical_key: PhysicalKey::KeyV, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "x", physical_key: PhysicalKey::KeyX, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "z", physical_key: PhysicalKey::KeyZ, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "q", physical_key: PhysicalKey::KeyQ, shift: false, alt_gr: false, initial_unlocked: false },
];

const HI_INSCRIPT_SEQUENCE: &[UnlockSpec] = &[
    // Home Row (PRD §2.2: ो े ् ि ु प र क त च ट)
    UnlockSpec { key_id: "ो", physical_key: PhysicalKey::KeyA, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "े", physical_key: PhysicalKey::KeyS, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "्", physical_key: PhysicalKey::KeyD, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ि", physical_key: PhysicalKey::KeyF, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ु", physical_key: PhysicalKey::KeyG, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "प", physical_key: PhysicalKey::KeyH, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "र", physical_key: PhysicalKey::KeyJ, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "क", physical_key: PhysicalKey::KeyK, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "त", physical_key: PhysicalKey::KeyL, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "च", physical_key: PhysicalKey::Semicolon, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ट", physical_key: PhysicalKey::Quote, shift: false, alt_gr: false, initial_unlocked: true },
    // Progressive Tiers
    UnlockSpec { key_id: "ा", physical_key: PhysicalKey::KeyE, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ी", physical_key: PhysicalKey::KeyR, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "न", physical_key: PhysicalKey::KeyV, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "म", physical_key: PhysicalKey::KeyC, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "स", physical_key: PhysicalKey::KeyM, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ल", physical_key: PhysicalKey::KeyN, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ह", physical_key: PhysicalKey::KeyU, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ब", physical_key: PhysicalKey::KeyY, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ग", physical_key: PhysicalKey::KeyI, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "द", physical_key: PhysicalKey::KeyO, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ज", physical_key: PhysicalKey::KeyP, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "व", physical_key: PhysicalKey::KeyB, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "य", physical_key: PhysicalKey::Slash, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ं", physical_key: PhysicalKey::KeyX, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ू", physical_key: PhysicalKey::KeyT, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ै", physical_key: PhysicalKey::KeyW, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ौ", physical_key: PhysicalKey::KeyQ, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ड", physical_key: PhysicalKey::BracketLeft, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "अ", physical_key: PhysicalKey::KeyD, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "आ", physical_key: PhysicalKey::KeyE, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "इ", physical_key: PhysicalKey::KeyF, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ई", physical_key: PhysicalKey::KeyR, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "उ", physical_key: PhysicalKey::KeyG, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ए", physical_key: PhysicalKey::KeyS, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ख", physical_key: PhysicalKey::KeyK, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "थ", physical_key: PhysicalKey::KeyL, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "फ", physical_key: PhysicalKey::KeyH, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "भ", physical_key: PhysicalKey::KeyY, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ध", physical_key: PhysicalKey::KeyO, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "घ", physical_key: PhysicalKey::KeyI, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "श", physical_key: PhysicalKey::KeyM, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ष", physical_key: PhysicalKey::Comma, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ण", physical_key: PhysicalKey::KeyC, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "छ", physical_key: PhysicalKey::Semicolon, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ठ", physical_key: PhysicalKey::Quote, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "झ", physical_key: PhysicalKey::KeyP, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ढ", physical_key: PhysicalKey::BracketLeft, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ञ", physical_key: PhysicalKey::BracketRight, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ृ", physical_key: PhysicalKey::Equal, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ओ", physical_key: PhysicalKey::KeyA, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ऐ", physical_key: PhysicalKey::KeyW, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "औ", physical_key: PhysicalKey::KeyQ, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ऊ", physical_key: PhysicalKey::KeyT, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ँ", physical_key: PhysicalKey::KeyX, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "़", physical_key: PhysicalKey::BracketRight, shift: false, alt_gr: false, initial_unlocked: false },
];

const HI_REMINGTON_SEQUENCE: &[UnlockSpec] = &[
    // Home Row (10 keys: ं े क ि ह ी र ा स य)
    UnlockSpec { key_id: "ं", physical_key: PhysicalKey::KeyA, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "े", physical_key: PhysicalKey::KeyS, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "क", physical_key: PhysicalKey::KeyD, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ि", physical_key: PhysicalKey::KeyF, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ह", physical_key: PhysicalKey::KeyG, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ी", physical_key: PhysicalKey::KeyH, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "र", physical_key: PhysicalKey::KeyJ, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "ा", physical_key: PhysicalKey::KeyK, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "स", physical_key: PhysicalKey::KeyL, shift: false, alt_gr: false, initial_unlocked: true },
    UnlockSpec { key_id: "य", physical_key: PhysicalKey::Semicolon, shift: false, alt_gr: false, initial_unlocked: true },
    // Progressive Tiers
    UnlockSpec { key_id: "म", physical_key: PhysicalKey::KeyE, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "त", physical_key: PhysicalKey::KeyR, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "न", physical_key: PhysicalKey::KeyU, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ल", physical_key: PhysicalKey::KeyY, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "प", physical_key: PhysicalKey::KeyI, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "व", physical_key: PhysicalKey::KeyO, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ज", physical_key: PhysicalKey::KeyT, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "च", physical_key: PhysicalKey::KeyP, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "द", physical_key: PhysicalKey::KeyN, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ग", physical_key: PhysicalKey::KeyX, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ब", physical_key: PhysicalKey::KeyC, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ु", physical_key: PhysicalKey::KeyQ, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ू", physical_key: PhysicalKey::KeyW, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "अ", physical_key: PhysicalKey::KeyV, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "इ", physical_key: PhysicalKey::KeyB, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "उ", physical_key: PhysicalKey::KeyM, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ए", physical_key: PhysicalKey::Comma, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ै", physical_key: PhysicalKey::KeyS, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "्र", physical_key: PhysicalKey::KeyZ, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "र्", physical_key: PhysicalKey::KeyZ, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "क्", physical_key: PhysicalKey::KeyD, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "त्", physical_key: PhysicalKey::KeyR, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "म्", physical_key: PhysicalKey::KeyE, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "स्", physical_key: PhysicalKey::KeyL, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "न्", physical_key: PhysicalKey::KeyU, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "श्", physical_key: PhysicalKey::Quote, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "थ्", physical_key: PhysicalKey::KeyF, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "भ्", physical_key: PhysicalKey::KeyH, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ध्", physical_key: PhysicalKey::Slash, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ख्", physical_key: PhysicalKey::BracketLeft, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ष्", physical_key: PhysicalKey::Quote, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ण्", physical_key: PhysicalKey::Period, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "घ्", physical_key: PhysicalKey::Slash, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "फ", physical_key: PhysicalKey::KeyQ, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ट", physical_key: PhysicalKey::KeyV, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ठ", physical_key: PhysicalKey::KeyB, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ड", physical_key: PhysicalKey::KeyM, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ढ", physical_key: PhysicalKey::Comma, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "छ", physical_key: PhysicalKey::KeyN, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ज्ञ", physical_key: PhysicalKey::KeyK, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "क्ष्", physical_key: PhysicalKey::BracketLeft, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "्", physical_key: PhysicalKey::Equal, shift: true, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ृ", physical_key: PhysicalKey::Equal, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "़", physical_key: PhysicalKey::Backquote, shift: false, alt_gr: false, initial_unlocked: false },
    UnlockSpec { key_id: "ँ", physical_key: PhysicalKey::Comma, shift: false, alt_gr: true, initial_unlocked: false },
];

pub fn unlock_sequence(layout: LayoutKind) -> &'static [UnlockSpec] {
    match layout {
        LayoutKind::EnQwerty => EN_QWERTY_SEQUENCE,
        LayoutKind::HiInscript => HI_INSCRIPT_SEQUENCE,
        LayoutKind::HiRemingtonGail => HI_REMINGTON_SEQUENCE,
    }
}

/// Builds default `KeyStatRecord` rows (Home Row unlocked, remaining keys locked).
pub fn default_key_stats(layout: LayoutKind) -> Vec<KeyStatRecord> {
    let now = Utc::now().to_rfc3339();
    unlock_sequence(layout)
        .iter()
        .enumerate()
        .map(|(idx, spec)| KeyStatRecord {
            layout,
            key_id: spec.key_id.to_string(),
            physical_code: spec.physical_key.as_dom_code().to_string(),
            requires_shift: spec.shift,
            requires_alt_gr: spec.alt_gr,
            unlock_order: idx as u32,
            unlocked: spec.initial_unlocked,
            hits: 0,
            misses: 0,
            ema_latency_ms: 0.0,
            confidence_score: 0.0,
            weakness_weight: 1.0,
            last_practiced_at: None,
            updated_at: now.clone(),
        })
        .collect()
}

/// Ensures persisted stats match the current layout's unlock sequence (preserving recorded telemetry).
pub fn reconcile_key_stats(layout: LayoutKind, existing: Vec<KeyStatRecord>) -> Vec<KeyStatRecord> {
    let mut by_id: HashMap<String, KeyStatRecord> = existing
        .into_iter()
        .map(|r| (r.key_id.clone(), r))
        .collect();

    let now = Utc::now().to_rfc3339();
    unlock_sequence(layout)
        .iter()
        .enumerate()
        .map(|(idx, spec)| {
            if let Some(mut found) = by_id.remove(spec.key_id) {
                found.unlock_order = idx as u32;
                found.physical_code = spec.physical_key.as_dom_code().to_string();
                found.requires_shift = spec.shift;
                found.requires_alt_gr = spec.alt_gr;
                if spec.initial_unlocked {
                    found.unlocked = true;
                }
                found
            } else {
                KeyStatRecord {
                    layout,
                    key_id: spec.key_id.to_string(),
                    physical_code: spec.physical_key.as_dom_code().to_string(),
                    requires_shift: spec.shift,
                    requires_alt_gr: spec.alt_gr,
                    unlock_order: idx as u32,
                    unlocked: spec.initial_unlocked,
                    hits: 0,
                    misses: 0,
                    ema_latency_ms: 0.0,
                    confidence_score: 0.0,
                    weakness_weight: 1.0,
                    last_practiced_at: None,
                    updated_at: now.clone(),
                }
            }
        })
        .collect()
}

/// Decomposes a dictionary word into the canonical `key_id` sequence required to type it.
pub fn required_keys_for_word(layout: LayoutKind, word: &str) -> Option<Vec<String>> {
    let norm = normalize_nfc(word);
    match layout {
        LayoutKind::EnQwerty => {
            let mut out = Vec::with_capacity(norm.len());
            for ch in norm.chars() {
                if !ch.is_ascii_alphabetic() {
                    return None;
                }
                out.push(ch.to_ascii_lowercase().to_string());
            }
            if out.is_empty() { None } else { Some(out) }
        }
        LayoutKind::HiInscript => {
            let valid: HashSet<&str> = HI_INSCRIPT_SEQUENCE.iter().map(|s| s.key_id).collect();
            let mut out = Vec::new();
            for ch in norm.chars() {
                if let Some((base, nukta)) = split_nukta_scalar(ch) {
                    if valid.contains(base) && valid.contains(nukta) {
                        out.push(base.to_string());
                        out.push(nukta.to_string());
                        continue;
                    }
                    return None;
                }
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                if valid.contains(s) {
                    out.push(s.to_string());
                } else {
                    return None;
                }
            }
            if out.is_empty() { None } else { Some(out) }
        }
        LayoutKind::HiRemingtonGail => decompose_remington_word(&norm),
    }
}

fn decompose_remington_word(word: &str) -> Option<Vec<String>> {
    let valid: HashSet<&str> = HI_REMINGTON_SEQUENCE.iter().map(|s| s.key_id).collect();
    let chars: Vec<char> = word.chars().collect();
    let mut keys = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        // Check 3-char conjuncts: ज्ञ, क्ष (+ ा), त्र, श्र
        if i + 2 < chars.len() && chars[i + 1] == '\u{094D}' {
            let tri: String = chars[i..=i + 2].iter().collect();
            if tri == "ज्ञ" && valid.contains("ज्ञ") {
                keys.push("ज्ञ".to_string());
                i += 3;
                continue;
            }
            if tri == "क्ष" {
                keys.push("क्ष्".to_string());
                keys.push("ा".to_string());
                i += 3;
                continue;
            }
            // Reph: र + ् + Consonant -> Consonant + र्
            if chars[i] == 'र' && is_consonant(chars[i + 2]) {
                let mut buf = [0u8; 4];
                let cons = chars[i + 2].encode_utf8(&mut buf);
                push_remington_consonant(cons, &valid, &mut keys)?;
                keys.push("र्".to_string());
                i += 3;
                continue;
            }
            // Rakar: Consonant + ् + र -> Consonant + ्र
            if is_consonant(chars[i]) && chars[i + 2] == 'र' {
                let mut buf = [0u8; 4];
                let cons = chars[i].encode_utf8(&mut buf);
                push_remington_consonant(cons, &valid, &mut keys)?;
                keys.push("्र".to_string());
                i += 3;
                continue;
            }
        }

        // Check 2-char half-consonant (`C + ्`)
        if i + 1 < chars.len() && chars[i + 1] == '\u{094D}' && is_consonant(chars[i]) {
            let half: String = chars[i..=i + 1].iter().collect();
            if valid.contains(half.as_str()) {
                keys.push(half);
            } else {
                let mut buf = [0u8; 4];
                let cons = chars[i].encode_utf8(&mut buf);
                push_remington_consonant(cons, &valid, &mut keys)?;
                keys.push("्".to_string());
            }
            i += 2;
            continue;
        }

        match chars[i] {
            'आ' => {
                keys.push("अ".to_string());
                keys.push("ा".to_string());
            }
            'ई' => {
                keys.push("इ".to_string());
                keys.push("र्".to_string());
            }
            'ऊ' => {
                keys.push("उ".to_string());
                keys.push("ु".to_string());
            }
            'ऐ' => {
                keys.push("ए".to_string());
                keys.push("े".to_string());
            }
            'ओ' => {
                keys.push("अ".to_string());
                keys.push("ा".to_string());
                keys.push("े".to_string());
            }
            'औ' => {
                keys.push("अ".to_string());
                keys.push("ा".to_string());
                keys.push("ै".to_string());
            }
            'ो' => {
                keys.push("ा".to_string());
                keys.push("े".to_string());
            }
            'ौ' => {
                keys.push("ा".to_string());
                keys.push("ै".to_string());
            }
            other => {
                if let Some((base, nukta)) = split_nukta_scalar(other) {
                    push_remington_consonant(base, &valid, &mut keys)?;
                    if valid.contains(nukta) {
                        keys.push(nukta.to_string());
                        i += 1;
                        continue;
                    }
                    return None;
                }
                let mut buf = [0u8; 4];
                let s = other.encode_utf8(&mut buf);
                if is_consonant(other) {
                    push_remington_consonant(s, &valid, &mut keys)?;
                } else if valid.contains(s) {
                    keys.push(s.to_string());
                } else {
                    return None;
                }
            }
        }
        i += 1;
    }

    if keys.is_empty() { None } else { Some(keys) }
}

fn push_remington_consonant(
    cons: &str,
    valid: &HashSet<&str>,
    out: &mut Vec<String>,
) -> Option<()> {
    if valid.contains(cons) {
        out.push(cons.to_string());
        return Some(());
    }
    let half = format!("{cons}\u{094D}");
    if valid.contains(half.as_str()) {
        out.push(half);
        out.push("ा".to_string());
        return Some(());
    }
    None
}

const fn is_consonant(c: char) -> bool {
    matches!(c, '\u{0915}'..='\u{0939}' | '\u{0958}'..='\u{095F}')
}

const fn split_nukta_scalar(c: char) -> Option<(&'static str, &'static str)> {
    match c {
        '\u{0929}' => Some(("न", "़")),
        '\u{0931}' => Some(("र", "़")),
        '\u{0934}' => Some(("ळ", "़")),
        '\u{0958}' => Some(("क", "़")),
        '\u{0959}' => Some(("ख", "़")),
        '\u{095A}' => Some(("ग", "़")),
        '\u{095B}' => Some(("ज", "़")),
        '\u{095C}' => Some(("ड", "़")),
        '\u{095D}' => Some(("ढ", "़")),
        '\u{095E}' => Some(("फ", "़")),
        '\u{095F}' => Some(("य", "़")),
        _ => None,
    }
}

/// Computes `weakness_weight` (70% error rate + 30% normalized latency capped at 2500ms)
/// and `confidence_score` (0..100).
pub fn recompute_scores(stat: &mut KeyStatRecord, settings: &AppSettings) {
    let total = stat.hits + stat.misses;
    if total == 0 {
        stat.weakness_weight = 1.0;
        stat.confidence_score = 0.0;
        return;
    }

    let error_rate = (stat.misses as f64 / total as f64).clamp(0.0, 1.0);
    let target_ms = 12_000.0 / settings.target_wpm.max(10.0);
    let fast_ms = target_ms * 0.6;
    let slow_ms = (target_ms * 3.0).min(MAX_LATENCY_CAP_MS);
    let clamped_lat = stat.ema_latency_ms.clamp(0.0, MAX_LATENCY_CAP_MS);
    let norm_latency = ((clamped_lat - fast_ms) / (slow_ms - fast_ms).max(100.0)).clamp(0.0, 1.0);

    stat.weakness_weight = ((error_rate * 0.7) + (norm_latency * 0.3)).clamp(0.0, 1.0);
    stat.confidence_score = (100.0 * (1.0 - stat.weakness_weight)).clamp(0.0, 100.0);
}

/// Updates telemetry for `key_id` after a keystroke and unlocks the next key if all active keys pass.
pub fn record_telemetry_and_check_unlock(
    stats: &mut [KeyStatRecord],
    key_id: &str,
    latency_ms: Option<f64>,
    is_correct: bool,
    settings: &AppSettings,
    reachable_keys: Option<&HashSet<String>>,
) -> Option<UnlockedKeyEvent> {
    let now = Utc::now().to_rfc3339();
    if let Some(stat) = stats.iter_mut().find(|s| s.key_id == key_id) {
        if is_correct {
            stat.hits += 1;
            if let Some(raw_lat) = latency_ms {
                let capped = raw_lat.clamp(25.0, MAX_LATENCY_CAP_MS);
                if stat.ema_latency_ms <= f64::EPSILON {
                    stat.ema_latency_ms = capped;
                } else {
                    stat.ema_latency_ms =
                        EMA_ALPHA * capped + (1.0 - EMA_ALPHA) * stat.ema_latency_ms;
                }
            }
        } else {
            stat.misses += 1;
        }
        stat.last_practiced_at = Some(now.clone());
        stat.updated_at = now.clone();
        recompute_scores(stat, settings);
    }

    try_unlock_next_key(stats, settings, reachable_keys)
}

/// Checks whether all reachable unlocked keys satisfy sample count, accuracy, and confidence threshold.
pub fn try_unlock_next_key(
    stats: &mut [KeyStatRecord],
    settings: &AppSettings,
    reachable_keys: Option<&HashSet<String>>,
) -> Option<UnlockedKeyEvent> {
    let all_mastered = stats.iter().filter(|s| s.unlocked).all(|s| {
        if let Some(reachable) = reachable_keys
            && !reachable.contains(&s.key_id)
            && (s.hits + s.misses == 0)
        {
            return true;
        }
        let total = s.hits + s.misses;
        if total < MIN_SAMPLES_FOR_UNLOCK {
            return false;
        }
        let acc = (s.hits as f64 / total as f64) * 100.0;
        s.confidence_score >= settings.unlock_threshold && acc >= settings.target_accuracy
    });

    if !all_mastered {
        return None;
    }

    let total_keys = stats.len() as u32;
    let next = stats
        .iter_mut()
        .filter(|s| !s.unlocked)
        .min_by_key(|s| s.unlock_order)?;

    next.unlocked = true;
    next.weakness_weight = 1.0;
    next.confidence_score = 0.0;
    next.updated_at = Utc::now().to_rfc3339();

    let finger = PhysicalKey::from_dom_code(&next.physical_code)
        .map(PhysicalKey::finger)
        .unwrap_or(FingerId::Thumb);

    Some(UnlockedKeyEvent {
        layout: next.layout,
        key_id: next.key_id.clone(),
        physical_code: next.physical_code.clone(),
        requires_shift: next.requires_shift,
        requires_alt_gr: next.requires_alt_gr,
        finger,
        unlock_order: next.unlock_order,
        total_keys,
    })
}

/// Filters `corpus` strictly to words composed solely of unlocked keys, and generates a
/// `words_per_line` lesson weighted 60% toward the top 3 weakest keys and 40% random unlocked words.
pub fn generate_adaptive_line(
    layout: LayoutKind,
    stats: &[KeyStatRecord],
    corpus: &Corpus,
    words_per_line: usize,
    rng: &mut impl Rng,
) -> Result<Vec<String>, AksharError> {
    let mut ranked: Vec<&KeyStatRecord> = stats.iter().filter(|s| s.unlocked).collect();
    ranked.sort_by(|a, b| {
        let a_unseen = a.hits + a.misses == 0;
        let b_unseen = b.hits + b.misses == 0;
        b_unseen
            .cmp(&a_unseen)
            .then_with(|| b.weakness_weight.total_cmp(&a.weakness_weight))
    });
    let top3_weak: HashSet<&str> = ranked.iter().take(3).map(|s| s.key_id.as_str()).collect();
    generate_weighted_line(layout, stats, corpus, words_per_line, &top3_weak, 0.6, rng)
}

/// Generates a targeted drill line heavily weighted (85%) toward `focus_keys`.
pub fn generate_weak_drill_line(
    layout: LayoutKind,
    stats: &[KeyStatRecord],
    focus_keys: &[String],
    corpus: &Corpus,
    words_per_line: usize,
    rng: &mut impl Rng,
) -> Result<Vec<String>, AksharError> {
    let focus: HashSet<&str> = focus_keys.iter().map(String::as_str).collect();
    generate_weighted_line(layout, stats, corpus, words_per_line, &focus, 0.85, rng)
}

fn generate_weighted_line(
    layout: LayoutKind,
    stats: &[KeyStatRecord],
    corpus: &Corpus,
    words_per_line: usize,
    focus_keys: &HashSet<&str>,
    weak_ratio: f64,
    rng: &mut impl Rng,
) -> Result<Vec<String>, AksharError> {
    let unlocked: HashSet<&str> = stats
        .iter()
        .filter(|s| s.unlocked)
        .map(|s| s.key_id.as_str())
        .collect();

    let filtered: Vec<(&str, Vec<String>)> = corpus
        .words_for(layout)
        .iter()
        .filter_map(|word| {
            let keys = required_keys_for_word(layout, word)?;
            if keys.iter().all(|k| unlocked.contains(k.as_str())) {
                Some((word.as_str(), keys))
            } else {
                None
            }
        })
        .collect();

    if filtered.is_empty() {
        return Err(AksharError::EmptyCorpus(layout.to_string()));
    }

    let mut weak_pool: Vec<&str> = Vec::new();
    let mut general_pool: Vec<&str> = Vec::with_capacity(filtered.len());

    for (word, keys) in &filtered {
        general_pool.push(*word);
        let weak_hits = keys.iter().filter(|k| focus_keys.contains(k.as_str())).count();
        if weak_hits > 0 {
            for _ in 0..weak_hits.min(3) {
                weak_pool.push(*word);
            }
        }
    }

    let weak_target = ((words_per_line as f64) * weak_ratio).round() as usize;
    let mut chosen = Vec::with_capacity(words_per_line);

    for _ in 0..weak_target {
        let pool = if weak_pool.is_empty() {
            &general_pool
        } else {
            &weak_pool
        };
        chosen.push(pick_non_repeating(pool, chosen.last().map(String::as_str), rng));
    }

    while chosen.len() < words_per_line {
        chosen.push(pick_non_repeating(
            &general_pool,
            chosen.last().map(String::as_str),
            rng,
        ));
    }

    chosen.shuffle(rng);
    if general_pool.len() > 1 {
        for idx in 1..chosen.len() {
            if chosen[idx] == chosen[idx - 1] {
                chosen[idx] = pick_non_repeating(&general_pool, Some(&chosen[idx - 1]), rng);
            }
        }
    }

    Ok(chosen)
}

fn pick_non_repeating(pool: &[&str], prev: Option<&str>, rng: &mut impl Rng) -> String {
    if pool.len() <= 1 {
        return pool.first().copied().unwrap_or("").to_string();
    }
    for _ in 0..6 {
        let pick = pool[rng.random_range(0..pool.len())];
        if Some(pick) != prev {
            return pick.to_string();
        }
    }
    pool[rng.random_range(0..pool.len())].to_string()
}

/// Maps a target grapheme & active layout to the canonical `key_id` for telemetry tracking.
pub fn primary_key_id_for_target(layout: LayoutKind, target_grapheme: &str) -> Option<String> {
    let keys = required_keys_for_word(layout, target_grapheme)?;
    keys.into_iter().next()
}

/// Computes the set of `key_id`s actually reachable in the corpus given currently unlocked keys.
pub fn reachable_unlocked_keys(
    layout: LayoutKind,
    stats: &[KeyStatRecord],
    corpus: &Corpus,
) -> HashSet<String> {
    let unlocked: HashSet<&str> = stats
        .iter()
        .filter(|s| s.unlocked)
        .map(|s| s.key_id.as_str())
        .collect();
    let mut reachable = HashSet::new();
    for word in corpus.words_for(layout) {
        if let Some(keys) = required_keys_for_word(layout, word)
            && keys.iter().all(|k| unlocked.contains(k.as_str()))
        {
            for k in keys {
                reachable.insert(k);
            }
        }
    }
    reachable
}
