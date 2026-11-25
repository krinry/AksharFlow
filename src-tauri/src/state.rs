use std::path::Path;
use std::sync::Mutex;

use aksharflow_core::error::AksharError;
use aksharflow_core::session::SessionEngine;

/// Shared thread-safe state managed by the Tauri v2 host.
pub struct AppState {
    pub engine: Mutex<SessionEngine>,
}

impl AppState {
    pub fn open(db_path: &Path) -> Result<Self, AksharError> {
        Ok(Self {
            engine: Mutex::new(SessionEngine::open_db(db_path)?),
        })
    }
}
