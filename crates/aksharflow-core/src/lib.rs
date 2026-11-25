pub mod adaptive;
pub mod corpus;
pub mod error;
pub mod grapheme;
pub mod keymap;
pub mod metrics;
pub mod session;
pub mod storage;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use crate::adaptive::{
        MAX_LATENCY_CAP_MS, MIN_SAMPLES_FOR_UNLOCK, default_key_stats, generate_adaptive_line,
        recompute_scores, record_telemetry_and_check_unlock, required_keys_for_word,
        unlock_sequence,
    };
    use crate::corpus::Corpus;
    use crate::grapheme::{GraphemeStatus, split_graphemes};
    use crate::keymap::remington_gail::RemingtonIme;
    use crate::keymap::{KeyInputEvent, LayoutKind, MappedKeyAction, PhysicalKey, qwerty};
    use crate::session::{KeySoundKind, SessionEngine};
    use crate::storage::{AppSettings, CustomPassage, SessionRecord, SqliteStorage};

    fn key_evt(code: &str, shift: bool, caps_lock: bool, ts: f64) -> KeyInputEvent {
        KeyInputEvent {
            code: code.to_string(),
            shift,
            alt_gr: false,
            ctrl: false,
            caps_lock,
            timestamp_ms: ts,
        }
    }

    #[test]
    fn qwerty_respects_caps_lock_only_on_letters() {
        assert_eq!(
            qwerty::map_key(PhysicalKey::KeyA, false, true),
            MappedKeyAction::Insert("A".to_string())
        );
        assert_eq!(
            qwerty::map_key(PhysicalKey::KeyA, true, true),
            MappedKeyAction::Insert("a".to_string())
        );
        assert_eq!(
            qwerty::map_key(PhysicalKey::Digit1, false, true),
            MappedKeyAction::Insert("1".to_string())
        );
    }

    #[test]
    fn hindi_layouts_ignore_caps_lock_safeguard() {
        let mut engine = SessionEngine::new().unwrap();
        let snap = engine
            .start_with_words(LayoutKind::HiInscript, vec!["कर".to_string()])
            .unwrap();
        assert_eq!(snap.graphemes, vec!["क", "र"]);

        let d1 = engine.process_key(&key_evt("KeyK", false, true, 100.0)).unwrap();
        assert_eq!(d1.grapheme_index, 1);
        assert_eq!(d1.committed_updates[0].status, GraphemeStatus::Matched);
    }

    #[test]
    fn remington_ime_visual_short_i_and_conjunct_reordering() {
        let mut ime = RemingtonIme::new();
        let (_, buf) = ime.process_key(PhysicalKey::KeyF, false, false, "");
        assert_eq!(ime.pending_preview(), Some("ि"));
        assert_eq!(buf, "");

        let (_, buf) = ime.process_key(PhysicalKey::KeyD, true, false, &buf);
        assert_eq!(buf, "क्");
        assert_eq!(ime.pending_preview(), Some("ि"));

        let (_, buf) = ime.process_key(PhysicalKey::KeyR, false, false, &buf);
        assert_eq!(buf, "क्ति");
        assert_eq!(ime.pending_preview(), None);
    }

    #[test]
    fn remington_ime_space_flushes_dangling_short_i_safeguard() {
        let mut ime = RemingtonIme::new();
        let (_, buf) = ime.process_key(PhysicalKey::KeyF, false, false, "");
        assert_eq!(ime.pending_preview(), Some("ि"));

        let (act, _) = ime.process_key(PhysicalKey::Space, false, false, &buf);
        assert_eq!(act, MappedKeyAction::FlushSpace);
        assert_eq!(ime.pending_preview(), None);

        let (_, next_buf) = ime.process_key(PhysicalKey::KeyD, false, false, "");
        assert_eq!(next_buf, "क");
    }

    #[test]
    fn remington_ime_reph_and_virama_strip() {
        let mut ime = RemingtonIme::new();
        let (_, buf) = ime.process_key(PhysicalKey::KeyF, true, false, "");
        let (_, buf) = ime.process_key(PhysicalKey::KeyK, false, false, &buf);
        assert_eq!(buf, "थ");

        ime.reset();
        let (_, buf) = ime.process_key(PhysicalKey::KeyB, false, false, "");
        let (_, buf) = ime.process_key(PhysicalKey::KeyZ, true, false, &buf);
        assert_eq!(buf, "ई");

        ime.reset();
        let (_, buf) = ime.process_key(PhysicalKey::KeyD, false, false, "");
        let (_, buf) = ime.process_key(PhysicalKey::KeyE, false, false, &buf);
        let (_, buf) = ime.process_key(PhysicalKey::KeyZ, true, false, &buf);
        assert_eq!(buf, "कर्म");
    }

    #[test]
    fn grapheme_splitter_groups_devanagari_clusters() {
        assert_eq!(split_graphemes("क्ति"), vec!["क्ति"]);
        assert_eq!(split_graphemes("अक्षर"), vec!["अ", "क्ष", "र"]);
        assert_eq!(split_graphemes("द्वार"), vec!["द्वा", "र"]);
    }

    #[test]
    fn session_engine_lightweight_delta_and_line_rollover() {
        let mut engine = SessionEngine::new().unwrap();
        let snap = engine
            .start_with_words(LayoutKind::EnQwerty, vec!["hi".to_string(), "ok".to_string()])
            .unwrap();
        assert_eq!(snap.graphemes, vec!["h", "i", " ", "o", "k"]);

        let d1 = engine.process_key(&key_evt("KeyH", false, false, 1000.0)).unwrap();
        assert_eq!(d1.grapheme_index, 1);
        assert!(!d1.line_completed);
        assert!(d1.next_line.is_none());

        let d2 = engine.process_key(&key_evt("KeyX", false, false, 1200.0)).unwrap();
        assert_eq!(d2.grapheme_index, 2);
        assert_eq!(d2.committed_updates[0].status, GraphemeStatus::Error);
        assert_eq!(d2.sound_kind, KeySoundKind::Error);

        let d_bs = engine.process_key(&key_evt("Backspace", false, false, 1350.0)).unwrap();
        assert_eq!(d_bs.grapheme_index, 1);
        assert_eq!(d_bs.committed_updates[0].status, GraphemeStatus::Pending);

        engine.process_key(&key_evt("KeyI", false, false, 1500.0)).unwrap();
        engine.process_key(&key_evt("Space", false, false, 1650.0)).unwrap();
        engine.process_key(&key_evt("KeyO", false, false, 1800.0)).unwrap();
        let d_end = engine.process_key(&key_evt("KeyK", false, false, 1950.0)).unwrap();
        assert!(d_end.line_completed);
        assert!(d_end.next_line.is_some());
        assert_eq!(d_end.next_line.unwrap().line_index, 1);
        assert_eq!(d_end.grapheme_index, 0);
        assert!(d_end.metrics.gross_wpm > 0.0);
    }

    #[test]
    fn remington_session_space_flush_end_to_end() {
        let mut engine = SessionEngine::new().unwrap();
        engine
            .start_with_words(
                LayoutKind::HiRemingtonGail,
                vec!["कि".to_string(), "कर".to_string()],
            )
            .unwrap();

        let d1 = engine.process_key(&key_evt("KeyF", false, false, 100.0)).unwrap();
        assert_eq!(d1.pending_ime_preview.as_deref(), Some("ि"));

        let d_sp = engine.process_key(&key_evt("Space", false, false, 200.0)).unwrap();
        assert_eq!(d_sp.pending_ime_preview, None);
        assert_eq!(d_sp.grapheme_index, 2);

        let d_ka = engine.process_key(&key_evt("KeyD", false, false, 350.0)).unwrap();
        assert_eq!(d_ka.grapheme_index, 3);
    }

    #[test]
    fn adaptive_weakness_weight_formula_and_latency_cap() {
        let settings = AppSettings::default(); // 25 WPM, 95% Acc, 85 threshold
        let mut stats = default_key_stats(LayoutKind::EnQwerty);
        let a = &mut stats[0];
        a.hits = 19;
        a.misses = 1; // 5% error rate
        a.ema_latency_ms = 480.0; // exact 25 WPM
        recompute_scores(a, &settings);
        assert!(a.confidence_score >= 85.0, "expected >= 85, got {}", a.confidence_score);

        // Extremely slow latency (>2500ms) is capped at 2500ms
        a.hits = 20;
        a.misses = 0;
        a.ema_latency_ms = 9999.0;
        recompute_scores(a, &settings);
        let expected_max_lat_weakness = 0.30; // 0 error * 0.7 + 1.0 * 0.3
        assert!((a.weakness_weight - expected_max_lat_weakness).abs() < 0.001);
        assert!(MAX_LATENCY_CAP_MS == 2500.0);
    }

    #[test]
    fn adaptive_progressive_unlock_triggers_when_home_row_mastered() {
        let settings = AppSettings::default();
        let mut stats = default_key_stats(LayoutKind::EnQwerty);
        let initial_unlocked: Vec<String> = stats
            .iter()
            .filter(|s| s.unlocked)
            .map(|s| s.key_id.clone())
            .collect();
        assert_eq!(initial_unlocked, vec!["a", "s", "d", "f", "j", "k", "l"]);
        assert!(!stats[7].unlocked); // 'e' locked initially

        let mut unlocked_evt = None;
        for key in &initial_unlocked {
            for _ in 0..MIN_SAMPLES_FOR_UNLOCK {
                unlocked_evt = record_telemetry_and_check_unlock(
                    &mut stats,
                    key,
                    Some(320.0),
                    true,
                    &settings,
                    None,
                );
            }
        }
        let evt = unlocked_evt.expect("expected next key 'e' to unlock once home row mastered");
        assert_eq!(evt.key_id, "e");
        assert!(stats[7].unlocked);
    }

    #[test]
    fn adaptive_word_generator_filters_strictly_and_weights_top3_weak_keys() {
        let corpus = Corpus::load_embedded().unwrap();
        let mut stats = default_key_stats(LayoutKind::EnQwerty);
        // Unlock 'e' and 'n' alongside Home Row, make 'e', 'j', 'k' clearly weakest
        for s in &mut stats {
            if matches!(s.key_id.as_str(), "e" | "n") {
                s.unlocked = true;
            }
            if s.unlocked {
                s.hits = 30;
                s.misses = 0;
                s.ema_latency_ms = 260.0;
                s.weakness_weight = 0.05;
                s.confidence_score = 95.0;
            }
            if matches!(s.key_id.as_str(), "e" | "j" | "k") {
                s.misses = 15;
                s.weakness_weight = 0.75;
                s.confidence_score = 25.0;
            }
        }

        let unlocked_set: HashSet<&str> = stats
            .iter()
            .filter(|s| s.unlocked)
            .map(|s| s.key_id.as_str())
            .collect();
        let mut r = StdRng::seed_from_u64(42);
        let words = generate_adaptive_line(LayoutKind::EnQwerty, &stats, &corpus, 20, &mut r).unwrap();
        assert_eq!(words.len(), 20);

        let mut weak_hits = 0usize;
        for w in &words {
            let keys = required_keys_for_word(LayoutKind::EnQwerty, w).unwrap();
            assert!(
                keys.iter().all(|k| unlocked_set.contains(k.as_str())),
                "word '{}' used a locked key!",
                w
            );
            if keys.iter().any(|k| matches!(k.as_str(), "e" | "j" | "k")) {
                weak_hits += 1;
            }
        }
        // At least 60% (12 of 20) should cover weak keys (plus some of the 40% random pool naturally also hit 'e')
        assert!(weak_hits >= 12, "expected >= 12 weak-key words, got {weak_hits}");
    }

    #[test]
    fn all_layouts_generate_valid_words_at_home_row_and_progressively() {
        let corpus = Corpus::load_embedded().unwrap();
        let mut r = StdRng::seed_from_u64(7);
        for layout in [
            LayoutKind::EnQwerty,
            LayoutKind::HiInscript,
            LayoutKind::HiRemingtonGail,
        ] {
            let mut stats = default_key_stats(layout);
            // Verify Home Row (tier 0) generates full 18-word line
            let line0 = generate_adaptive_line(layout, &stats, &corpus, 18, &mut r).unwrap();
            assert_eq!(line0.len(), 18, "tier 0 failed for {layout}");

            // Progressively unlock every key and verify line generation succeeds at every tier
            let total = unlock_sequence(layout).len();
            for idx in 0..total {
                stats[idx].unlocked = true;
                let line = generate_adaptive_line(layout, &stats, &corpus, 18, &mut r).unwrap();
                assert_eq!(line.len(), 18, "tier {idx} failed for {layout}");
            }
        }
    }

    #[test]
    fn sqlite_storage_and_backup_roundtrip() {
        let mut storage = SqliteStorage::open_in_memory().unwrap();
        let mut settings = storage.load_or_init_settings().unwrap();
        settings.target_wpm = 35.0;
        settings.ui_lang = "hi".to_string();
        storage.save_settings(&settings).unwrap();

        let mut stats = default_key_stats(LayoutKind::HiInscript);
        stats[0].hits = 25;
        stats[0].misses = 1;
        stats[0].ema_latency_ms = 390.0;
        recompute_scores(&mut stats[0], &settings);
        storage.upsert_key_stats(&stats).unwrap();

        storage
            .record_typing_session(&SessionRecord {
                id: "sess-1".to_string(),
                layout: LayoutKind::HiInscript,
                mode: "adaptive".to_string(),
                duration_ms: 42_000,
                gross_wpm: 38.5,
                net_wpm: 36.2,
                kph: 10860.0,
                accuracy: 96.4,
                consistency: 88.1,
                total_chars: 135,
                errors: 3,
                created_at: "2026-11-25T15:50:00Z".to_string(),
            })
            .unwrap();

        storage
            .save_custom_passage(&CustomPassage {
                id: "psg-1".to_string(),
                language: "hi".to_string(),
                title: "CPCT Practice".to_string(),
                content: "अभ्यास से ही सिद्धि मिलती है।".to_string(),
                created_at: "2026-11-25T15:50:00Z".to_string(),
            })
            .unwrap();

        let backup = storage.export_backup().unwrap();
        let json = serde_json::to_string(&backup).unwrap();
        let parsed: crate::storage::BackupBundleV1 = serde_json::from_str(&json).unwrap();

        let mut fresh = SqliteStorage::open_in_memory().unwrap();
        fresh.import_backup(&parsed, true).unwrap();

        let loaded_settings = fresh.load_or_init_settings().unwrap();
        assert_eq!(loaded_settings.target_wpm, 35.0);
        assert_eq!(loaded_settings.ui_lang, "hi");

        let loaded_stats = fresh.load_key_stats(LayoutKind::HiInscript).unwrap();
        assert_eq!(loaded_stats[0].hits, 25);
        assert_eq!(fresh.list_typing_sessions(None, 10).unwrap().len(), 1);
        assert_eq!(fresh.list_custom_passages().unwrap().len(), 1);
    }
}
