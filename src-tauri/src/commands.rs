use std::path::Path;

use tauri::State;

use aksharflow_core::keymap::{KeyInputEvent, KeycapDescriptor, LayoutKind, layout_keycaps};
use aksharflow_core::session::{
    AnalyticsSnapshot, BootstrapSnapshot, ExportResult, KeyEventDelta, SessionSummary,
};
use aksharflow_core::storage::{AppSettings, CustomPassage};

use crate::state::AppState;

/// Loads persisted settings, per-key stats, recent sessions, and starts the initial adaptive line.
#[tauri::command]
#[specta::specta]
pub fn get_bootstrap_state(
    state: State<'_, AppState>,
    words_per_line: Option<u32>,
) -> Result<BootstrapSnapshot, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .bootstrap(words_per_line.map(|n| n as usize))
        .map_err(|e| e.to_string())
}

/// Starts or switches to an adaptive typing session for `layout` and returns the bootstrap snapshot.
#[tauri::command]
#[specta::specta]
pub fn start_session(
    state: State<'_, AppState>,
    layout: LayoutKind,
    words_per_line: Option<u32>,
) -> Result<BootstrapSnapshot, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .start_session(layout, words_per_line.map(|n| n as usize))
        .map_err(|e| e.to_string())?;
    engine
        .bootstrap(words_per_line.map(|n| n as usize))
        .map_err(|e| e.to_string())
}

/// Processes a physical or touch-keyboard key event and returns only a lightweight state delta.
#[tauri::command]
#[specta::specta]
pub fn process_key_event(
    state: State<'_, AppState>,
    event: KeyInputEvent,
) -> Result<KeyEventDelta, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine.process_key(&event).map_err(|e| e.to_string())
}

/// Ends the active typing session, persists summary telemetry, and returns metrics.
#[tauri::command]
#[specta::specta]
pub fn finish_session(state: State<'_, AppState>) -> Result<SessionSummary, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine.finish_session().map_err(|e| e.to_string())
}

/// Returns per-key accuracy/latency telemetry and recent session history for `layout`.
#[tauri::command]
#[specta::specta]
pub fn get_analytics(
    state: State<'_, AppState>,
    layout: LayoutKind,
) -> Result<AnalyticsSnapshot, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine.analytics(layout).map_err(|e| e.to_string())
}

/// Persists updated settings and recomputes per-key confidence scores.
#[tauri::command]
#[specta::specta]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine.update_settings(settings).map_err(|e| e.to_string())
}

/// Exports settings, key stats, sessions, and custom passages as JSON to `path`.
#[tauri::command]
#[specta::specta]
pub fn export_backup_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<ExportResult, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .export_backup_to_file(Path::new(&path))
        .map_err(|e| e.to_string())
}

/// Imports and validates a `.json` backup file (`replace = true` overwrites, `false` merges).
#[tauri::command]
#[specta::specta]
pub fn import_backup_file(
    state: State<'_, AppState>,
    path: String,
    replace: bool,
) -> Result<BootstrapSnapshot, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .import_backup_from_file(Path::new(&path), replace)
        .map_err(|e| e.to_string())
}

/// Resets unlocked keys and telemetry for `layout` back to Home Row.
#[tauri::command]
#[specta::specta]
pub fn reset_layout_progress(
    state: State<'_, AppState>,
    layout: LayoutKind,
) -> Result<BootstrapSnapshot, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .reset_layout_progress(layout)
        .map_err(|e| e.to_string())
}

/// Saves a custom practice/exam passage to SQLite.
#[tauri::command]
#[specta::specta]
pub fn save_custom_passage(
    state: State<'_, AppState>,
    title: String,
    language: String,
    content: String,
) -> Result<CustomPassage, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine
        .save_custom_passage(title, language, content)
        .map_err(|e| e.to_string())
}

/// Lists saved custom passages.
#[tauri::command]
#[specta::specta]
pub fn list_custom_passages(state: State<'_, AppState>) -> Result<Vec<CustomPassage>, String> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| "Engine state lock poisoned".to_string())?;
    engine.list_custom_passages().map_err(|e| e.to_string())
}

/// Returns virtual keyboard keycap descriptors for the requested layout.
#[tauri::command]
#[specta::specta]
pub fn get_layout_keycaps(layout: LayoutKind) -> Vec<KeycapDescriptor> {
    layout_keycaps(layout)
}
