use std::sync::Arc;

use cc_switch::{AppState, Database};
use cc_switch_core::{import_config_from_sql_bytes, CoreContext};

#[test]
fn failed_sql_upload_does_not_pollute_existing_database() {
    let db = Arc::new(Database::memory().expect("in-memory database"));
    let before = db.export_sql_string().expect("export before");
    let ctx = CoreContext::from_app_state(AppState::new(db.clone()));

    let invalid_sql = "-- CC Switch SQLite 导出\nTHIS IS NOT VALID SQL;";
    let err = import_config_from_sql_bytes(&ctx, invalid_sql.as_bytes())
        .expect_err("invalid SQL should fail");

    assert!(!err.is_empty());
    let after = db.export_sql_string().expect("export after");
    assert_eq!(before, after, "failed import should not mutate existing data");
}
