use std::path::Path;

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::error::AksharError;
use crate::keymap::LayoutKind;

pub const BACKUP_SCHEMA_VERSION: u32 = 1;

/// Persisted application & adaptive learning preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct AppSettings {
    pub ui_lang: String,
    pub theme: String,
    pub sound_profile: String,
    pub sound_volume: f64,
    pub active_layout: LayoutKind,
    pub target_wpm: f64,
    pub target_accuracy: f64,
    pub unlock_threshold: f64,
    pub zen_mode: bool,
    pub updated_at: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_lang: "en".to_string(),
            theme: "dark".to_string(),
            sound_profile: "brown".to_string(),
            sound_volume: 0.55,
            active_layout: LayoutKind::EnQwerty,
            target_wpm: 25.0,
            target_accuracy: 95.0,
            unlock_threshold: 85.0,
            zen_mode: false,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Persisted per-key telemetry and unlock state for a layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct KeyStatRecord {
    pub layout: LayoutKind,
    pub key_id: String,
    pub physical_code: String,
    pub requires_shift: bool,
    pub requires_alt_gr: bool,
    pub unlock_order: u32,
    pub unlocked: bool,
    pub hits: u32,
    pub misses: u32,
    pub ema_latency_ms: f64,
    pub confidence_score: f64,
    pub weakness_weight: f64,
    pub last_practiced_at: Option<String>,
    pub updated_at: String,
}

/// Persisted completed typing session summary row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct SessionRecord {
    pub id: String,
    pub layout: LayoutKind,
    pub mode: String,
    pub duration_ms: u32,
    pub gross_wpm: f64,
    pub net_wpm: f64,
    pub kph: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub total_chars: u32,
    pub errors: u32,
    pub created_at: String,
}

/// User-saved custom practice or exam passage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct CustomPassage {
    pub id: String,
    pub language: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

/// Portable JSON backup archive for user progress without cloud login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta-types", derive(specta::Type))]
pub struct BackupBundleV1 {
    pub schema_version: u32,
    pub app_version: String,
    pub exported_at: String,
    pub settings: AppSettings,
    pub key_stats: Vec<KeyStatRecord>,
    pub typing_sessions: Vec<SessionRecord>,
    pub custom_passages: Vec<CustomPassage>,
}

/// Embedded SQLite storage repository using WAL mode for fast offline writes.
#[derive(Debug)]
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Opens (or creates) an on-disk SQLite database at `path` with WAL mode and runs migrations.
    pub fn open(path: &Path) -> Result<Self, AksharError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.configure_pragmas(true)?;
        storage.migrate()?;
        Ok(storage)
    }

    /// Opens an isolated in-memory SQLite database for automated testing.
    pub fn open_in_memory() -> Result<Self, AksharError> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.configure_pragmas(false)?;
        storage.migrate()?;
        Ok(storage)
    }

    fn configure_pragmas(&self, enable_wal: bool) -> Result<(), AksharError> {
        if enable_wal {
            self.conn.pragma_update(None, "journal_mode", "WAL")?;
            self.conn.pragma_update(None, "synchronous", "NORMAL")?;
        }
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(())
    }

    fn migrate(&self) -> Result<(), AksharError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS app_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                ui_lang TEXT NOT NULL,
                theme TEXT NOT NULL,
                sound_profile TEXT NOT NULL,
                sound_volume REAL NOT NULL,
                active_layout TEXT NOT NULL,
                target_wpm REAL NOT NULL,
                target_accuracy REAL NOT NULL,
                unlock_threshold REAL NOT NULL,
                zen_mode INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS key_stats (
                layout TEXT NOT NULL,
                key_id TEXT NOT NULL,
                physical_code TEXT NOT NULL,
                requires_shift INTEGER NOT NULL DEFAULT 0,
                requires_alt_gr INTEGER NOT NULL DEFAULT 0,
                unlock_order INTEGER NOT NULL,
                unlocked INTEGER NOT NULL,
                hits INTEGER NOT NULL DEFAULT 0,
                misses INTEGER NOT NULL DEFAULT 0,
                ema_latency_ms REAL NOT NULL DEFAULT 0.0,
                confidence_score REAL NOT NULL DEFAULT 0.0,
                weakness_weight REAL NOT NULL DEFAULT 1.0,
                last_practiced_at TEXT,
                updated_at TEXT NOT NULL,
                sync_dirty INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (layout, key_id)
            );

            CREATE INDEX IF NOT EXISTS idx_key_stats_layout_order
                ON key_stats (layout, unlock_order);

            CREATE TABLE IF NOT EXISTS typing_sessions (
                id TEXT PRIMARY KEY,
                layout TEXT NOT NULL,
                mode TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                gross_wpm REAL NOT NULL,
                net_wpm REAL NOT NULL,
                kph REAL NOT NULL,
                accuracy REAL NOT NULL,
                consistency REAL NOT NULL,
                total_chars INTEGER NOT NULL,
                errors INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                sync_dirty INTEGER NOT NULL DEFAULT 1
            );

            CREATE INDEX IF NOT EXISTS idx_typing_sessions_layout_created
                ON typing_sessions (layout, created_at DESC);

            CREATE TABLE IF NOT EXISTS custom_passages (
                id TEXT PRIMARY KEY,
                language TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn load_or_init_settings(&self) -> Result<AppSettings, AksharError> {
        let existing = self
            .conn
            .query_row(
                "SELECT ui_lang, theme, sound_profile, sound_volume, active_layout,
                        target_wpm, target_accuracy, unlock_threshold, zen_mode, updated_at
                 FROM app_settings WHERE id = 1",
                [],
                |row| {
                    let layout_str: String = row.get(4)?;
                    Ok(AppSettings {
                        ui_lang: row.get(0)?,
                        theme: row.get(1)?,
                        sound_profile: row.get(2)?,
                        sound_volume: row.get(3)?,
                        active_layout: parse_layout(&layout_str).unwrap_or(LayoutKind::EnQwerty),
                        target_wpm: row.get(5)?,
                        target_accuracy: row.get(6)?,
                        unlock_threshold: row.get(7)?,
                        zen_mode: row.get::<_, i64>(8)? != 0,
                        updated_at: row.get(9)?,
                    })
                },
            )
            .optional()?;

        if let Some(settings) = existing {
            Ok(settings)
        } else {
            let defaults = AppSettings::default();
            self.save_settings(&defaults)?;
            Ok(defaults)
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<AppSettings, AksharError> {
        let normalized = sanitize_settings(settings.clone());
        self.conn.execute(
            "INSERT INTO app_settings (
                id, ui_lang, theme, sound_profile, sound_volume, active_layout,
                target_wpm, target_accuracy, unlock_threshold, zen_mode, updated_at
             ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                ui_lang = excluded.ui_lang,
                theme = excluded.theme,
                sound_profile = excluded.sound_profile,
                sound_volume = excluded.sound_volume,
                active_layout = excluded.active_layout,
                target_wpm = excluded.target_wpm,
                target_accuracy = excluded.target_accuracy,
                unlock_threshold = excluded.unlock_threshold,
                zen_mode = excluded.zen_mode,
                updated_at = excluded.updated_at",
            params![
                normalized.ui_lang,
                normalized.theme,
                normalized.sound_profile,
                normalized.sound_volume,
                normalized.active_layout.to_string(),
                normalized.target_wpm,
                normalized.target_accuracy,
                normalized.unlock_threshold,
                if normalized.zen_mode { 1 } else { 0 },
                normalized.updated_at,
            ],
        )?;
        Ok(normalized)
    }

    pub fn load_key_stats(&self, layout: LayoutKind) -> Result<Vec<KeyStatRecord>, AksharError> {
        let mut stmt = self.conn.prepare(
            "SELECT layout, key_id, physical_code, requires_shift, requires_alt_gr,
                    unlock_order, unlocked, hits, misses, ema_latency_ms,
                    confidence_score, weakness_weight, last_practiced_at, updated_at
             FROM key_stats
             WHERE layout = ?1
             ORDER BY unlock_order ASC",
        )?;
        let rows = stmt.query_map([layout.to_string()], map_key_stat_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn load_all_key_stats(&self) -> Result<Vec<KeyStatRecord>, AksharError> {
        let mut stmt = self.conn.prepare(
            "SELECT layout, key_id, physical_code, requires_shift, requires_alt_gr,
                    unlock_order, unlocked, hits, misses, ema_latency_ms,
                    confidence_score, weakness_weight, last_practiced_at, updated_at
             FROM key_stats
             ORDER BY layout ASC, unlock_order ASC",
        )?;
        let rows = stmt.query_map([], map_key_stat_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_key_stats(&mut self, records: &[KeyStatRecord]) -> Result<(), AksharError> {
        let tx = self.conn.transaction()?;
        upsert_key_stats_tx(&tx, records)?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_key_stats_for_layout(&self, layout: LayoutKind) -> Result<(), AksharError> {
        self.conn.execute(
            "DELETE FROM key_stats WHERE layout = ?1",
            [layout.to_string()],
        )?;
        Ok(())
    }

    pub fn record_typing_session(&self, session: &SessionRecord) -> Result<(), AksharError> {
        self.conn.execute(
            "INSERT INTO typing_sessions (
                id, layout, mode, duration_ms, gross_wpm, net_wpm, kph,
                accuracy, consistency, total_chars, errors, created_at, sync_dirty
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1)
             ON CONFLICT(id) DO UPDATE SET
                duration_ms = excluded.duration_ms,
                gross_wpm = excluded.gross_wpm,
                net_wpm = excluded.net_wpm,
                kph = excluded.kph,
                accuracy = excluded.accuracy,
                consistency = excluded.consistency,
                total_chars = excluded.total_chars,
                errors = excluded.errors,
                sync_dirty = 1",
            params![
                session.id,
                session.layout.to_string(),
                session.mode,
                session.duration_ms,
                session.gross_wpm,
                session.net_wpm,
                session.kph,
                session.accuracy,
                session.consistency,
                session.total_chars,
                session.errors,
                session.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_typing_sessions(
        &self,
        layout: Option<LayoutKind>,
        limit: usize,
    ) -> Result<Vec<SessionRecord>, AksharError> {
        let lim = limit.clamp(1, 500) as i64;
        let mut out = Vec::new();
        if let Some(target_layout) = layout {
            let mut stmt = self.conn.prepare(
                "SELECT id, layout, mode, duration_ms, gross_wpm, net_wpm, kph,
                        accuracy, consistency, total_chars, errors, created_at
                 FROM typing_sessions
                 WHERE layout = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![target_layout.to_string(), lim], map_session_row)?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, layout, mode, duration_ms, gross_wpm, net_wpm, kph,
                        accuracy, consistency, total_chars, errors, created_at
                 FROM typing_sessions
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![lim], map_session_row)?;
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    }

    pub fn save_custom_passage(&self, passage: &CustomPassage) -> Result<(), AksharError> {
        self.conn.execute(
            "INSERT INTO custom_passages (id, language, title, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                language = excluded.language,
                title = excluded.title,
                content = excluded.content",
            params![
                passage.id,
                passage.language,
                passage.title,
                passage.content,
                passage.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_custom_passages(&self) -> Result<Vec<CustomPassage>, AksharError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, language, title, content, created_at
             FROM custom_passages
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CustomPassage {
                id: row.get(0)?,
                language: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Exports all user settings, per-key stats, session history, and custom passages as `BackupBundleV1`.
    pub fn export_backup(&self) -> Result<BackupBundleV1, AksharError> {
        Ok(BackupBundleV1 {
            schema_version: BACKUP_SCHEMA_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: Utc::now().to_rfc3339(),
            settings: self.load_or_init_settings()?,
            key_stats: self.load_all_key_stats()?,
            typing_sessions: self.list_typing_sessions(None, 500)?,
            custom_passages: self.list_custom_passages()?,
        })
    }

    /// Exports pretty-printed JSON directly to `path`.
    pub fn export_backup_to_file(&self, path: &Path) -> Result<BackupBundleV1, AksharError> {
        let bundle = self.export_backup()?;
        let json = serde_json::to_string_pretty(&bundle)?;
        std::fs::write(path, json)?;
        Ok(bundle)
    }

    /// Validates and imports a `BackupBundleV1` JSON file atomically in a transaction.
    pub fn import_backup_from_file(
        &mut self,
        path: &Path,
        replace: bool,
    ) -> Result<BackupBundleV1, AksharError> {
        let raw = std::fs::read_to_string(path)?;
        let bundle: BackupBundleV1 = serde_json::from_str(&raw)?;
        self.import_backup(&bundle, replace)?;
        Ok(bundle)
    }

    /// Validates and applies a `BackupBundleV1` to SQLite (`replace = true` overwrites, `false` merges).
    pub fn import_backup(
        &mut self,
        bundle: &BackupBundleV1,
        replace: bool,
    ) -> Result<(), AksharError> {
        validate_backup(bundle)?;
        let settings = sanitize_settings(bundle.settings.clone());

        let tx = self.conn.transaction()?;
        if replace {
            tx.execute("DELETE FROM key_stats", [])?;
            tx.execute("DELETE FROM typing_sessions", [])?;
            tx.execute("DELETE FROM custom_passages", [])?;
        }

        tx.execute(
            "INSERT INTO app_settings (
                id, ui_lang, theme, sound_profile, sound_volume, active_layout,
                target_wpm, target_accuracy, unlock_threshold, zen_mode, updated_at
             ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                ui_lang = excluded.ui_lang,
                theme = excluded.theme,
                sound_profile = excluded.sound_profile,
                sound_volume = excluded.sound_volume,
                active_layout = excluded.active_layout,
                target_wpm = excluded.target_wpm,
                target_accuracy = excluded.target_accuracy,
                unlock_threshold = excluded.unlock_threshold,
                zen_mode = excluded.zen_mode,
                updated_at = excluded.updated_at",
            params![
                settings.ui_lang,
                settings.theme,
                settings.sound_profile,
                settings.sound_volume,
                settings.active_layout.to_string(),
                settings.target_wpm,
                settings.target_accuracy,
                settings.unlock_threshold,
                if settings.zen_mode { 1 } else { 0 },
                settings.updated_at,
            ],
        )?;

        if replace {
            upsert_key_stats_tx(&tx, &bundle.key_stats)?;
        } else {
            for rec in &bundle.key_stats {
                let existing: Option<(bool, u32, u32, f64, f64, f64)> = tx
                    .query_row(
                        "SELECT unlocked, hits, misses, ema_latency_ms, confidence_score, weakness_weight
                         FROM key_stats WHERE layout = ?1 AND key_id = ?2",
                        params![rec.layout.to_string(), rec.key_id],
                        |row| {
                            Ok((
                                row.get::<_, i64>(0)? != 0,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                                row.get(5)?,
                            ))
                        },
                    )
                    .optional()?;

                let merged = if let Some((old_unlocked, old_hits, old_misses, old_lat, old_conf, old_weak)) = existing {
                    let old_total = old_hits + old_misses;
                    let new_total = rec.hits + rec.misses;
                    let use_new = new_total > old_total;
                    KeyStatRecord {
                        unlocked: old_unlocked || rec.unlocked,
                        hits: old_hits.max(rec.hits),
                        misses: if use_new { rec.misses } else { old_misses },
                        ema_latency_ms: if use_new { rec.ema_latency_ms } else { old_lat },
                        confidence_score: if use_new { rec.confidence_score } else { old_conf },
                        weakness_weight: if use_new { rec.weakness_weight } else { old_weak },
                        ..rec.clone()
                    }
                } else {
                    rec.clone()
                };
                upsert_key_stats_tx(&tx, &[merged])?;
            }
        }

        for sess in &bundle.typing_sessions {
            tx.execute(
                "INSERT INTO typing_sessions (
                    id, layout, mode, duration_ms, gross_wpm, net_wpm, kph,
                    accuracy, consistency, total_chars, errors, created_at, sync_dirty
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    sess.id,
                    sess.layout.to_string(),
                    sess.mode,
                    sess.duration_ms,
                    sess.gross_wpm,
                    sess.net_wpm,
                    sess.kph,
                    sess.accuracy.clamp(0.0, 100.0),
                    sess.consistency.clamp(0.0, 100.0),
                    sess.total_chars,
                    sess.errors,
                    sess.created_at,
                ],
            )?;
        }

        for passage in &bundle.custom_passages {
            tx.execute(
                "INSERT INTO custom_passages (id, language, title, content, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    passage.id,
                    passage.language,
                    passage.title,
                    passage.content,
                    passage.created_at,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }
}

fn upsert_key_stats_tx(
    tx: &rusqlite::Transaction<'_>,
    records: &[KeyStatRecord],
) -> Result<(), AksharError> {
    let mut stmt = tx.prepare(
        "INSERT INTO key_stats (
            layout, key_id, physical_code, requires_shift, requires_alt_gr,
            unlock_order, unlocked, hits, misses, ema_latency_ms,
            confidence_score, weakness_weight, last_practiced_at, updated_at, sync_dirty
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 1)
         ON CONFLICT(layout, key_id) DO UPDATE SET
            physical_code = excluded.physical_code,
            requires_shift = excluded.requires_shift,
            requires_alt_gr = excluded.requires_alt_gr,
            unlock_order = excluded.unlock_order,
            unlocked = excluded.unlocked,
            hits = excluded.hits,
            misses = excluded.misses,
            ema_latency_ms = excluded.ema_latency_ms,
            confidence_score = excluded.confidence_score,
            weakness_weight = excluded.weakness_weight,
            last_practiced_at = excluded.last_practiced_at,
            updated_at = excluded.updated_at,
            sync_dirty = 1",
    )?;
    for r in records {
        stmt.execute(params![
            r.layout.to_string(),
            r.key_id,
            r.physical_code,
            if r.requires_shift { 1 } else { 0 },
            if r.requires_alt_gr { 1 } else { 0 },
            r.unlock_order,
            if r.unlocked { 1 } else { 0 },
            r.hits,
            r.misses,
            r.ema_latency_ms,
            r.confidence_score.clamp(0.0, 100.0),
            r.weakness_weight.clamp(0.0, 1.0),
            r.last_practiced_at,
            r.updated_at,
        ])?;
    }
    Ok(())
}

fn map_key_stat_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<KeyStatRecord> {
    let layout_str: String = row.get(0)?;
    Ok(KeyStatRecord {
        layout: parse_layout(&layout_str).unwrap_or(LayoutKind::EnQwerty),
        key_id: row.get(1)?,
        physical_code: row.get(2)?,
        requires_shift: row.get::<_, i64>(3)? != 0,
        requires_alt_gr: row.get::<_, i64>(4)? != 0,
        unlock_order: row.get(5)?,
        unlocked: row.get::<_, i64>(6)? != 0,
        hits: row.get(7)?,
        misses: row.get(8)?,
        ema_latency_ms: row.get(9)?,
        confidence_score: row.get(10)?,
        weakness_weight: row.get(11)?,
        last_practiced_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn map_session_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionRecord> {
    let layout_str: String = row.get(1)?;
    Ok(SessionRecord {
        id: row.get(0)?,
        layout: parse_layout(&layout_str).unwrap_or(LayoutKind::EnQwerty),
        mode: row.get(2)?,
        duration_ms: row.get(3)?,
        gross_wpm: row.get(4)?,
        net_wpm: row.get(5)?,
        kph: row.get(6)?,
        accuracy: row.get(7)?,
        consistency: row.get(8)?,
        total_chars: row.get(9)?,
        errors: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn parse_layout(s: &str) -> Option<LayoutKind> {
    match s {
        "en_qwerty" => Some(LayoutKind::EnQwerty),
        "hi_inscript" => Some(LayoutKind::HiInscript),
        "hi_remington_gail" => Some(LayoutKind::HiRemingtonGail),
        _ => None,
    }
}

fn sanitize_settings(mut s: AppSettings) -> AppSettings {
    s.ui_lang = if s.ui_lang == "hi" {
        "hi".to_string()
    } else {
        "en".to_string()
    };
    s.theme = if s.theme == "light" {
        "light".to_string()
    } else {
        "dark".to_string()
    };
    if !matches!(
        s.sound_profile.as_str(),
        "off" | "blue" | "brown" | "red" | "topre"
    ) {
        s.sound_profile = "brown".to_string();
    }
    s.sound_volume = s.sound_volume.clamp(0.0, 1.0);
    s.target_wpm = s.target_wpm.clamp(10.0, 140.0);
    s.target_accuracy = s.target_accuracy.clamp(70.0, 100.0);
    s.unlock_threshold = s.unlock_threshold.clamp(60.0, 99.0);
    s.updated_at = Utc::now().to_rfc3339();
    s
}

fn validate_backup(bundle: &BackupBundleV1) -> Result<(), AksharError> {
    if bundle.schema_version != BACKUP_SCHEMA_VERSION {
        return Err(AksharError::InvalidBackup(format!(
            "Unsupported backup schema version {} (expected {})",
            bundle.schema_version, BACKUP_SCHEMA_VERSION
        )));
    }
    for ks in &bundle.key_stats {
        if ks.key_id.trim().is_empty() || ks.physical_code.trim().is_empty() {
            return Err(AksharError::InvalidBackup(
                "KeyStatRecord contains empty key_id or physical_code".to_string(),
            ));
        }
    }
    for sess in &bundle.typing_sessions {
        if sess.id.trim().is_empty() {
            return Err(AksharError::InvalidBackup(
                "SessionRecord contains empty id".to_string(),
            ));
        }
    }
    Ok(())
}
