mod commands;
mod state;

use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{Builder, collect_commands};

use crate::commands::{
    export_backup_file, finish_session, get_analytics, get_bootstrap_state, get_layout_keycaps,
    import_backup_file, list_custom_passages, process_key_event, reset_layout_progress,
    save_custom_passage, start_session, update_settings,
};
use crate::state::AppState;

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        get_bootstrap_state,
        start_session,
        process_key_event,
        finish_session,
        get_analytics,
        update_settings,
        export_backup_file,
        import_backup_file,
        reset_layout_progress,
        save_custom_passage,
        list_custom_passages,
        get_layout_keycaps,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/lib/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve OS app data directory");
            let db_path = app_data.join("aksharflow.sqlite3");
            let app_state =
                AppState::open(&db_path).expect("Failed to open AksharFlow SQLite database");
            app.manage(app_state);
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running AksharFlow Tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_typescript_bindings() {
        specta_builder()
            .export(Typescript::default(), "../src/lib/bindings.ts")
            .expect("Failed to export TypeScript bindings");
    }
}
