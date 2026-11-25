use std::path::PathBuf;
use std::rc::Rc;

use aksharflow_core::session::SessionEngine;
use aksharflow_craftui::mount_aksharflow_app;
use craftui::prelude::*;

fn default_db_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("aksharflow").join("aksharflow-craftui.db")
}

fn main() -> craftui::core::Result<()> {
    run_app(
        WindowConfig {
            title: "AksharFlow (CraftUI)".to_string(),
            width: 1240.0,
            height: 820.0,
            ..Default::default()
        },
        |cx, tree, _app_context| {
            let db_path = default_db_path();
            let engine = SessionEngine::open_db(&db_path)
                .unwrap_or_else(|_| SessionEngine::new().expect("failed initializing engine"));
            let (root, core) = mount_aksharflow_app(cx, tree, engine);

            core.borrow_mut().export_hook = Some(Rc::new(|json| {
                let dlg = DesktopFileDialog;
                let req = FileDialogRequest::save(
                    "Export AksharFlow Progress",
                    "aksharflow-backup.json",
                )
                .with_filter(FileFilter::new("JSON", ["json"]));
                if let Some(path) = dlg.save_file(&req) {
                    std::fs::write(&path, json).map_err(|e| e.to_string())?;
                    Ok(path.display().to_string())
                } else {
                    Err("Export cancelled".to_string())
                }
            }));

            core.borrow_mut().import_hook = Some(Rc::new(|| {
                let dlg = DesktopFileDialog;
                let req = FileDialogRequest::open("Import AksharFlow Progress")
                    .with_filter(FileFilter::new("JSON", ["json"]));
                dlg.open_files(&req)
                    .into_iter()
                    .next()
                    .and_then(|path| std::fs::read_to_string(path).ok())
            }));

            root
        },
    )
}
