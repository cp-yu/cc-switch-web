use chrono::Local;
use serde_json::{json, Value};
use std::{fs, path::Path, sync::Arc};

use crate::database::Database;
use crate::error::AppError;
use crate::services::provider::ProviderService;
use crate::settings;
use crate::store::AppState;

pub fn default_sql_export_file_name() -> String {
    format!(
        "cc-switch-export-{}.sql",
        Local::now().format("%Y%m%d_%H%M%S")
    )
}

pub fn export_database_sql(db: Arc<Database>) -> Result<String, AppError> {
    db.export_sql_string()
}

pub fn export_database_to_file(db: Arc<Database>, target_path: &Path) -> Result<(), AppError> {
    let dump = export_database_sql(db)?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
    }

    crate::config::atomic_write(target_path, dump.as_bytes())
}

fn run_post_import_sync(db: Arc<Database>) -> Result<(), AppError> {
    let app_state = AppState::new(db);
    ProviderService::sync_current_to_live(&app_state)?;
    settings::reload_settings()?;
    Ok(())
}

fn post_sync_warning<E: std::fmt::Display>(err: E) -> String {
    AppError::localized(
        "sync.post_operation_sync_failed",
        format!("后置同步状态失败: {err}"),
        format!("Post-operation synchronization failed: {err}"),
    )
    .to_string()
}

fn post_sync_warning_from_result(result: Result<(), AppError>) -> Option<String> {
    match result {
        Ok(()) => None,
        Err(err) => Some(post_sync_warning(err)),
    }
}

fn success_payload_with_warning(backup_id: String, warning: Option<String>) -> Value {
    let mut payload = json!({
        "success": true,
        "message": "SQL imported successfully",
        "backupId": backup_id,
    });

    if let Some(message) = warning {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("warning".to_string(), Value::String(message));
        }
    }

    payload
}

pub fn import_database_with_sync<F>(db: Arc<Database>, import: F) -> Result<Value, AppError>
where
    F: FnOnce(&Database) -> Result<String, AppError>,
{
    let db_for_sync = db.clone();
    let backup_id = import(&db)?;
    let warning = post_sync_warning_from_result(run_post_import_sync(db_for_sync));
    if let Some(msg) = warning.as_ref() {
        log::warn!("[Import] post-import sync warning: {msg}");
    }
    Ok(success_payload_with_warning(backup_id, warning))
}
