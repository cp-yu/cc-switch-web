//! cc-switch-core
//!
//! 该 crate 提供与 UI 无关的核心业务封装，供 Web 服务器等复用。
//! 当前实现基于现有的 `cc_switch`（src-tauri）进行轻量封装，
//! 后续可以逐步将纯业务逻辑下沉到本 crate。

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use cc_switch::{
    AppError, AppSettings, AppState, AppType, Database, EndpointLatency, McpServer, Provider,
    ProviderService, SkillService, SpeedtestService,
};
use indexmap::IndexMap;

/// 对外暴露的核心类型别名，便于直接使用
pub use cc_switch::{
    AppSettings as CoreAppSettings, AppType as CoreAppType, DailyStats, HealthStatus,
    LogFilters, McpServer as CoreMcpServer, ModelPricingInfo as CoreModelPricingInfo,
    ModelStats, PaginatedLogs, Provider as CoreProvider, ProviderLimitStatus, ProviderStats,
    RequestLogDetail, StreamCheckConfig, StreamCheckResult, StreamCheckService, UsageSummary,
    WEB_COMPAT_TAURI_COMMANDS,
};

/// 核心上下文
///
/// - 管理共享的数据库连接
/// - 管理 SkillService 等长生命周期服务
pub struct CoreContext {
    app_state: AppState,
    skill_service: Option<Arc<SkillService>>,
}

impl CoreContext {
    /// 初始化核心上下文
    ///
    /// - 打开/初始化 `~/.cc-switch/cc-switch.db`
    /// - 构造 `AppState`
    /// - 尝试初始化 `SkillService`（失败时只记录为 None，不阻塞其它功能）
    pub fn new() -> Result<Self, AppError> {
        let db = Arc::new(Database::init()?);
        let app_state = AppState::new(db);

        let skill_service = Some(Arc::new(SkillService::new()));

        Ok(Self {
            app_state,
            skill_service,
        })
    }

    /// 获取应用状态（包含数据库）
    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }

    /// 获取 SkillService（如果初始化成功）
    pub fn skill_service(&self) -> Option<&Arc<SkillService>> {
        self.skill_service.as_ref()
    }
}

// ========================
// Provider 相关 API
// ========================

/// 获取指定应用下的所有供应商
pub fn get_providers(ctx: &CoreContext, app: &str) -> Result<IndexMap<String, Provider>, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::list(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

/// 获取指定应用的当前供应商 ID
pub fn get_current_provider(ctx: &CoreContext, app: &str) -> Result<String, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::current(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

/// 添加供应商
pub fn add_provider(
    ctx: &CoreContext,
    app: &str,
    provider: Provider,
) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::add(ctx.app_state(), app_type, provider).map_err(|e| e.to_string())
}

/// 更新供应商
pub fn update_provider(
    ctx: &CoreContext,
    app: &str,
    provider: Provider,
) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::update(ctx.app_state(), app_type, provider).map_err(|e| e.to_string())
}

/// 删除供应商
pub fn delete_provider(ctx: &CoreContext, app: &str, id: &str) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::delete(ctx.app_state(), app_type, id)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

/// 切换供应商
pub fn switch_provider(ctx: &CoreContext, app: &str, id: &str) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::switch(ctx.app_state(), app_type, id)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

/// 导入当前配置为默认供应商
pub fn import_default_config(ctx: &CoreContext, app: &str) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::import_default_config(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

/// 更新多个供应商的排序
pub fn update_providers_sort_order(
    ctx: &CoreContext,
    app: &str,
    updates: &serde_json::Value,
) -> Result<bool, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let updates = serde_json::from_value(updates.clone()).map_err(|e| e.to_string())?;
    ProviderService::update_sort_order(ctx.app_state(), app_type, updates)
        .map_err(|e| e.to_string())
}

/// 查询供应商用量
pub async fn query_provider_usage(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
) -> Result<serde_json::Value, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let result = ProviderService::query_usage(ctx.app_state(), app_type, provider_id)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// 测试用量脚本（使用当前编辑器中的脚本，不保存）
#[allow(clippy::too_many_arguments)]
pub async fn test_usage_script(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
    script_code: &str,
    timeout: Option<u64>,
    api_key: Option<&str>,
    base_url: Option<&str>,
    access_token: Option<&str>,
    user_id: Option<&str>,
    template_type: Option<&str>,
) -> Result<serde_json::Value, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let result = ProviderService::test_usage_script(
        ctx.app_state(),
        app_type,
        provider_id,
        script_code,
        timeout.unwrap_or(10),
        api_key,
        base_url,
        access_token,
        user_id,
        template_type,
    )
    .await
    .map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

/// 读取当前生效的配置内容
pub fn read_live_provider_settings(app: &str) -> Result<serde_json::Value, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::read_live_settings(app_type).map_err(|e| e.to_string())
}

/// 获取自定义端点列表
pub fn get_custom_endpoints(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
) -> Result<serde_json::Value, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let endpoints =
        ProviderService::get_custom_endpoints(ctx.app_state(), app_type, provider_id)
            .map_err(|e| e.to_string())?;
    serde_json::to_value(endpoints).map_err(|e| e.to_string())
}

/// 添加自定义端点
pub fn add_custom_endpoint(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::add_custom_endpoint(ctx.app_state(), app_type, provider_id, url)
        .map_err(|e| e.to_string())
}

/// 删除自定义端点
pub fn remove_custom_endpoint(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::remove_custom_endpoint(ctx.app_state(), app_type, provider_id, url)
        .map_err(|e| e.to_string())
}

/// 更新端点最后使用时间
pub fn update_endpoint_last_used(
    ctx: &CoreContext,
    app: &str,
    provider_id: &str,
    url: String,
) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::update_endpoint_last_used(ctx.app_state(), app_type, provider_id, url)
        .map_err(|e| e.to_string())
}

pub async fn stream_check_provider(
    ctx: &CoreContext,
    app_type: &str,
    provider_id: &str,
) -> Result<StreamCheckResult, String> {
    let app_type = AppType::from_str(app_type).map_err(|e| e.to_string())?;
    let config = ctx
        .app_state()
        .db
        .get_stream_check_config()
        .map_err(|e| e.to_string())?;

    let providers = ctx
        .app_state()
        .db
        .get_all_providers(app_type.as_str())
        .map_err(|e| e.to_string())?;
    let provider = providers
        .get(provider_id)
        .ok_or_else(|| format!("供应商 {provider_id} 不存在"))?;

    let result = StreamCheckService::check_with_retry(&app_type, provider, &config)
        .await
        .map_err(|e| e.to_string())?;

    let _ = ctx.app_state().db.save_stream_check_log(
        provider_id,
        &provider.name,
        app_type.as_str(),
        &result,
    );

    Ok(result)
}

pub async fn stream_check_all_providers(
    ctx: &CoreContext,
    app_type: &str,
    proxy_targets_only: bool,
) -> Result<Vec<(String, StreamCheckResult)>, String> {
    let app_type = AppType::from_str(app_type).map_err(|e| e.to_string())?;
    let config = ctx
        .app_state()
        .db
        .get_stream_check_config()
        .map_err(|e| e.to_string())?;
    let providers = ctx
        .app_state()
        .db
        .get_all_providers(app_type.as_str())
        .map_err(|e| e.to_string())?;

    let allowed_ids = if proxy_targets_only {
        let mut ids = std::collections::HashSet::new();
        if let Ok(Some(current_id)) = ctx.app_state().db.get_current_provider(app_type.as_str()) {
            ids.insert(current_id);
        }
        if let Ok(queue) = ctx.app_state().db.get_failover_queue(app_type.as_str()) {
            for item in queue {
                ids.insert(item.provider_id);
            }
        }
        Some(ids)
    } else {
        None
    };

    let mut results = Vec::new();
    for (id, provider) in providers {
        if let Some(ids) = &allowed_ids {
            if !ids.contains(&id) {
                continue;
            }
        }

        let result = StreamCheckService::check_with_retry(&app_type, &provider, &config)
            .await
            .unwrap_or_else(|e| StreamCheckResult {
                status: HealthStatus::Failed,
                success: false,
                message: e.to_string(),
                response_time_ms: None,
                http_status: None,
                model_used: String::new(),
                tested_at: Utc::now().timestamp(),
                retry_count: 0,
            });

        let _ = ctx
            .app_state()
            .db
            .save_stream_check_log(&id, &provider.name, app_type.as_str(), &result);

        results.push((id, result));
    }

    Ok(results)
}

pub fn get_stream_check_config(ctx: &CoreContext) -> Result<StreamCheckConfig, String> {
    ctx.app_state()
        .db
        .get_stream_check_config()
        .map_err(|e| e.to_string())
}

pub fn save_stream_check_config(
    ctx: &CoreContext,
    config: StreamCheckConfig,
) -> Result<(), String> {
    ctx.app_state()
        .db
        .save_stream_check_config(&config)
        .map_err(|e| e.to_string())
}

pub fn get_usage_summary(
    ctx: &CoreContext,
    start_date: Option<i64>,
    end_date: Option<i64>,
) -> Result<UsageSummary, String> {
    ctx.app_state()
        .db
        .get_usage_summary(start_date, end_date)
        .map_err(|e| e.to_string())
}

pub fn get_usage_trends(
    ctx: &CoreContext,
    start_date: Option<i64>,
    end_date: Option<i64>,
) -> Result<Vec<DailyStats>, String> {
    ctx.app_state()
        .db
        .get_daily_trends(start_date, end_date)
        .map_err(|e| e.to_string())
}

pub fn get_provider_stats(ctx: &CoreContext) -> Result<Vec<ProviderStats>, String> {
    ctx.app_state().db.get_provider_stats().map_err(|e| e.to_string())
}

pub fn get_model_stats(ctx: &CoreContext) -> Result<Vec<ModelStats>, String> {
    ctx.app_state().db.get_model_stats().map_err(|e| e.to_string())
}

pub fn get_request_logs(
    ctx: &CoreContext,
    filters: LogFilters,
    page: u32,
    page_size: u32,
) -> Result<PaginatedLogs, String> {
    ctx.app_state()
        .db
        .get_request_logs(&filters, page, page_size)
        .map_err(|e| e.to_string())
}

pub fn get_request_detail(
    ctx: &CoreContext,
    request_id: &str,
) -> Result<Option<RequestLogDetail>, String> {
    ctx.app_state()
        .db
        .get_request_detail(request_id)
        .map_err(|e| e.to_string())
}

pub fn get_model_pricing(ctx: &CoreContext) -> Result<Vec<CoreModelPricingInfo>, String> {
    cc_switch::list_model_pricing(&ctx.app_state().db).map_err(|e| e.to_string())
}

pub fn update_model_pricing(
    ctx: &CoreContext,
    model_id: String,
    display_name: String,
    input_cost: String,
    output_cost: String,
    cache_read_cost: String,
    cache_creation_cost: String,
) -> Result<(), String> {
    cc_switch::upsert_model_pricing(
        &ctx.app_state().db,
        model_id,
        display_name,
        input_cost,
        output_cost,
        cache_read_cost,
        cache_creation_cost,
    )
    .map_err(|e| e.to_string())
}

pub fn delete_model_pricing(ctx: &CoreContext, model_id: String) -> Result<(), String> {
    cc_switch::remove_model_pricing(&ctx.app_state().db, model_id).map_err(|e| e.to_string())
}

pub fn check_provider_limits(
    ctx: &CoreContext,
    provider_id: &str,
    app_type: &str,
) -> Result<ProviderLimitStatus, String> {
    ctx.app_state()
        .db
        .check_provider_limits(provider_id, app_type)
        .map_err(|e| e.to_string())
}

/// 测试第三方/自定义供应商端点的网络延迟
pub async fn test_api_endpoints(
    urls: Vec<String>,
    timeout_secs: Option<u64>,
) -> Result<Vec<EndpointLatency>, String> {
    SpeedtestService::test_endpoints(urls, timeout_secs)
        .await
        .map_err(|e| e.to_string())
}

/// Web / 服务器模式下更新托盘菜单（无操作，返回 true 以兼容前端调用）
pub fn update_tray_menu(_ctx: &CoreContext) -> Result<bool, String> {
    Ok(true)
}

// ========================
// Settings 相关 API
// ========================

/// 获取应用设置
pub fn get_settings() -> AppSettings {
    cc_switch::get_settings()
}

/// 保存应用设置
pub fn save_settings(settings: AppSettings) -> Result<bool, String> {
    cc_switch::update_settings(settings).map_err(|e| e.to_string())?;
    Ok(true)
}

/// 重启应用 (stub - not applicable for web server)
pub fn restart_app() -> Result<bool, String> {
    // Web server mode does not support app restart
    // Return true to indicate the request was received
    Ok(true)
}

/// 检查更新 (stub - not applicable for web server)
/// Returns the update URL for the client to handle
pub fn check_for_updates() -> Result<String, String> {
    Ok("https://github.com/farion1231/cc-switch/releases/latest".to_string())
}

/// 判断是否为便携版运行
pub fn is_portable_mode() -> Result<bool, String> {
    let exe_path = std::env::current_exe().map_err(|e| format!("获取可执行路径失败: {e}"))?;
    if let Some(dir) = exe_path.parent() {
        Ok(dir.join("portable.ini").is_file())
    } else {
        Ok(false)
    }
}

/// 获取配置目录路径
pub fn get_config_dir(app: &str) -> Result<String, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    let dir = match app_type {
        AppType::Claude => cc_switch::get_claude_config_dir(),
        AppType::Codex => cc_switch::get_codex_config_dir(),
        AppType::Gemini => cc_switch::get_gemini_dir(),
        AppType::OpenCode => cc_switch::get_opencode_dir(),
        AppType::OpenClaw => cc_switch::get_openclaw_dir(),
    };
    Ok(dir.to_string_lossy().to_string())
}

/// 打开配置文件夹 (stub - not applicable for web server)
/// Returns the path for the client to handle
pub fn open_config_folder(app: &str) -> Result<String, String> {
    get_config_dir(app)
}

/// 选择目录 (stub - not applicable for web server)
/// Returns None as directory picking requires native dialog
pub fn pick_directory() -> Result<Option<String>, String> {
    Ok(None)
}

/// 获取 Claude Code 配置文件路径
pub fn get_claude_code_config_path() -> String {
    cc_switch::get_claude_settings_path()
        .to_string_lossy()
        .to_string()
}

/// 获取应用配置文件路径
pub fn get_app_config_path() -> String {
    cc_switch::get_app_config_path()
        .to_string_lossy()
        .to_string()
}

/// 获取应用配置目录路径
pub fn get_app_config_dir() -> String {
    cc_switch::get_app_config_dir()
        .to_string_lossy()
        .to_string()
}

/// 打开应用配置文件夹 (stub - not applicable for web server)
/// Returns the path for the client to handle
pub fn open_app_config_folder() -> Result<String, String> {
    Ok(get_app_config_dir())
}

/// 获取 app_config_dir 覆盖配置
/// In web mode, we read from environment variable or return None
pub fn get_app_config_dir_override() -> Option<String> {
    std::env::var("CC_SWITCH_CONFIG_DIR").ok()
}

/// 设置 app_config_dir 覆盖配置 (stub - not fully applicable for web server)
/// In web mode, this would require server restart to take effect
pub fn set_app_config_dir_override(_path: Option<&str>) -> Result<bool, String> {
    // Web server mode does not support runtime config dir changes
    // The config dir is determined at startup
    Ok(true)
}

/// 应用 Claude 插件配置
pub fn apply_claude_plugin_config(official: bool) -> Result<bool, String> {
    if official {
        cc_switch::clear_claude_config().map_err(|e| e.to_string())
    } else {
        cc_switch::write_claude_config().map_err(|e| e.to_string())
    }
}

/// 保存文件对话框 (stub - not applicable for web server)
/// Returns None as file dialogs require native UI
pub fn save_file_dialog() -> Result<Option<String>, String> {
    Ok(None)
}

/// 打开文件对话框 (stub - not applicable for web server)
/// Returns None as file dialogs require native UI
pub fn open_file_dialog() -> Result<Option<String>, String> {
    Ok(None)
}

/// 导出配置到文件
pub fn export_config_to_file(ctx: &CoreContext, file_path: &str) -> Result<serde_json::Value, String> {
    let target_path = std::path::PathBuf::from(file_path);
    ctx.app_state()
        .db
        .export_sql(&target_path)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "message": "SQL exported successfully",
        "filePath": file_path
    }))
}

/// 从文件导入配置
pub fn import_config_from_file(ctx: &CoreContext, file_path: &str) -> Result<serde_json::Value, String> {
    let path_buf = std::path::PathBuf::from(file_path);
    let backup_id = ctx
        .app_state()
        .db
        .import_sql(&path_buf)
        .map_err(|e| e.to_string())?;

    // 导入后同步当前供应商到各自的 live 配置
    if let Err(err) = ProviderService::sync_current_to_live(ctx.app_state()) {
        log::warn!("导入后同步 live 配置失败: {err}");
    }

    // 重新加载设置到内存缓存
    if let Err(err) = cc_switch::reload_settings() {
        log::warn!("导入后重载设置失败: {err}");
    }

    Ok(serde_json::json!({
        "success": true,
        "message": "SQL imported successfully",
        "backupId": backup_id
    }))
}

/// 同步当前供应商到 live 配置
pub fn sync_current_providers_live(ctx: &CoreContext) -> Result<serde_json::Value, String> {
    ProviderService::sync_current_to_live(ctx.app_state()).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "message": "Live configuration synchronized"
    }))
}

/// 打开外部链接 (stub - not applicable for web server)
/// Returns the URL for the client to handle
pub fn open_external(url: &str) -> Result<String, String> {
    let url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    Ok(url)
}

/// 设置开机自启 (stub - not applicable for web server)
pub fn set_auto_launch(_enabled: bool) -> Result<bool, String> {
    // Web server mode does not support auto launch
    Ok(true)
}

/// 获取开机自启状态 (stub - returns false for web server)
pub fn get_auto_launch_status() -> Result<bool, String> {
    // Web server mode does not support auto launch
    Ok(false)
}

// ========================
// Skill 相关 API
// ========================

/// 获取所有技能（返回 JSON 值，避免直接依赖内部 Skill 类型）
pub async fn get_skills(ctx: &CoreContext) -> Result<serde_json::Value, String> {
    let service = ctx
        .skill_service()
        .ok_or_else(|| "SkillService 未初始化".to_string())?;

    let repos = ctx
        .app_state()
        .db
        .get_skill_repos()
        .map_err(|e| e.to_string())?;

    let skills = service
        .list_skills(repos, &ctx.app_state().db)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(skills).map_err(|e| e.to_string())
}

// ========================
// MCP 相关 API
// ========================

/// MCP 状态信息
pub use cc_switch::McpStatus;

/// 获取 Claude MCP 状态
pub fn get_claude_mcp_status() -> Result<McpStatus, String> {
    cc_switch::get_claude_mcp_status_raw().map_err(|e| e.to_string())
}

/// 读取 Claude MCP 配置文件内容
pub fn read_claude_mcp_config() -> Result<Option<String>, String> {
    cc_switch::read_claude_mcp_config_raw().map_err(|e| e.to_string())
}

/// 在 Claude MCP 配置中添加或更新服务器
pub fn upsert_claude_mcp_server(id: &str, spec: serde_json::Value) -> Result<bool, String> {
    cc_switch::upsert_claude_mcp_server_raw(id, spec).map_err(|e| e.to_string())
}

/// 在 Claude MCP 配置中删除服务器
pub fn delete_claude_mcp_server(id: &str) -> Result<bool, String> {
    cc_switch::delete_claude_mcp_server_raw(id).map_err(|e| e.to_string())
}

/// 校验命令是否在 PATH 中可用
pub fn validate_mcp_command(cmd: &str) -> Result<bool, String> {
    cc_switch::validate_mcp_command_raw(cmd).map_err(|e| e.to_string())
}

/// MCP 配置响应（用于兼容旧 API）
#[derive(serde::Serialize)]
pub struct McpConfigResponse {
    pub config_path: String,
    pub servers: std::collections::HashMap<String, serde_json::Value>,
}

/// 获取 MCP 配置（来自 ~/.cc-switch/config.json）
#[allow(deprecated)]
pub fn get_mcp_config(ctx: &CoreContext, app: &str) -> Result<McpConfigResponse, String> {
    let config_path = cc_switch::get_app_config_path()
        .to_string_lossy()
        .to_string();
    let app_ty = AppType::from_str(app).map_err(|e| e.to_string())?;
    let servers =
        cc_switch::McpService::get_servers(ctx.app_state(), app_ty).map_err(|e| e.to_string())?;
    Ok(McpConfigResponse {
        config_path,
        servers,
    })
}

/// 在 config.json 中新增或更新一个 MCP 服务器定义（兼容旧 API）
pub fn upsert_mcp_server_in_config(
    ctx: &CoreContext,
    app: &str,
    id: &str,
    spec: serde_json::Value,
    sync_other_side: Option<bool>,
) -> Result<bool, String> {
    use cc_switch::McpApps;

    let app_ty = AppType::from_str(app).map_err(|e| e.to_string())?;

    // 读取现有的服务器（如果存在）
    let existing_server = {
        let servers = ctx
            .app_state()
            .db
            .get_all_mcp_servers()
            .map_err(|e| e.to_string())?;
        servers.get(id).cloned()
    };

    // 构建新的统一服务器结构
    let mut new_server = if let Some(mut existing) = existing_server {
        // 更新现有服务器
        existing.server = spec.clone();
        existing.apps.set_enabled_for(&app_ty, true);
        existing
    } else {
        // 创建新服务器
        let mut apps = McpApps::default();
        apps.set_enabled_for(&app_ty, true);

        // 尝试从 spec 中提取 name，否则使用 id
        let name = spec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();

        McpServer {
            id: id.to_string(),
            name,
            server: spec,
            apps,
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        }
    };

    // 如果 sync_other_side 为 true，也启用其他应用
    if sync_other_side.unwrap_or(false) {
        new_server.apps.claude = true;
        new_server.apps.codex = true;
        new_server.apps.gemini = true;
    }

    cc_switch::McpService::upsert_server(ctx.app_state(), new_server)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

/// 在 config.json 中删除一个 MCP 服务器定义
pub fn delete_mcp_server_in_config(ctx: &CoreContext, id: &str) -> Result<bool, String> {
    cc_switch::McpService::delete_server(ctx.app_state(), id).map_err(|e| e.to_string())
}

/// 设置启用状态并同步到客户端配置
#[allow(deprecated)]
pub fn set_mcp_enabled(
    ctx: &CoreContext,
    app: &str,
    id: &str,
    enabled: bool,
) -> Result<bool, String> {
    let app_ty = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::McpService::set_enabled(ctx.app_state(), app_ty, id, enabled)
        .map_err(|e| e.to_string())
}

/// 获取所有 MCP 服务器（统一结构）
pub fn get_mcp_servers(ctx: &CoreContext) -> Result<IndexMap<String, McpServer>, String> {
    cc_switch::McpService::get_all_servers(ctx.app_state()).map_err(|e| e.to_string())
}

/// 添加或更新 MCP 服务器（统一结构）
pub fn upsert_mcp_server(ctx: &CoreContext, server: McpServer) -> Result<(), String> {
    cc_switch::McpService::upsert_server(ctx.app_state(), server).map_err(|e| e.to_string())
}

/// 删除 MCP 服务器
pub fn delete_mcp_server(ctx: &CoreContext, id: &str) -> Result<bool, String> {
    cc_switch::McpService::delete_server(ctx.app_state(), id).map_err(|e| e.to_string())
}

/// 切换 MCP 服务器在指定应用的启用状态
pub fn toggle_mcp_app(
    ctx: &CoreContext,
    server_id: &str,
    app: &str,
    enabled: bool,
) -> Result<(), String> {
    let app_ty = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::McpService::toggle_app(ctx.app_state(), server_id, app_ty, enabled)
        .map_err(|e| e.to_string())
}

// ========================
// Prompt 相关 API
// ========================

/// 导出 Prompt 类型
pub use cc_switch::Prompt;

/// 获取所有提示词
pub fn get_prompts(
    ctx: &CoreContext,
    app: &str,
) -> Result<IndexMap<String, cc_switch::Prompt>, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::get_prompts(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

/// 添加或更新提示词
pub fn upsert_prompt(
    ctx: &CoreContext,
    app: &str,
    id: &str,
    prompt: cc_switch::Prompt,
) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::upsert_prompt(ctx.app_state(), app_type, id, prompt)
        .map_err(|e| e.to_string())
}

/// 删除提示词
pub fn delete_prompt(ctx: &CoreContext, app: &str, id: &str) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::delete_prompt(ctx.app_state(), app_type, id)
        .map_err(|e| e.to_string())
}

/// 启用提示词
pub fn enable_prompt(ctx: &CoreContext, app: &str, id: &str) -> Result<(), String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::enable_prompt(ctx.app_state(), app_type, id)
        .map_err(|e| e.to_string())
}

/// 从文件导入提示词
pub fn import_prompt_from_file(ctx: &CoreContext, app: &str) -> Result<String, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::import_from_file(ctx.app_state(), app_type)
        .map_err(|e| e.to_string())
}

/// 获取当前提示词文件内容
pub fn get_current_prompt_file_content(app: &str) -> Result<Option<String>, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    cc_switch::PromptService::get_current_file_content(app_type).map_err(|e| e.to_string())
}

// ========================
// Environment 相关 API
// ========================

/// 导出环境变量相关类型
pub use cc_switch::{BackupInfo, EnvConflict};

/// 检查环境变量冲突
pub fn check_env_conflicts(app: &str) -> Result<Vec<cc_switch::EnvConflict>, String> {
    cc_switch::check_env_conflicts(app)
}

/// 删除环境变量（带备份）
pub fn delete_env_vars(
    conflicts: Vec<cc_switch::EnvConflict>,
) -> Result<cc_switch::BackupInfo, String> {
    cc_switch::delete_env_vars(conflicts)
}

/// 从备份恢复环境变量
pub fn restore_env_backup(backup_path: &str) -> Result<(), String> {
    cc_switch::restore_from_backup(backup_path.to_string())
}

// ========================
// Config Snippet 相关 API
// ========================

/// 获取 Claude 通用配置片段（已废弃，使用 get_common_config_snippet）
pub fn get_claude_common_config_snippet(ctx: &CoreContext) -> Result<Option<String>, String> {
    ctx.app_state()
        .db
        .get_config_snippet("claude")
        .map_err(|e| e.to_string())
}

/// 设置 Claude 通用配置片段（已废弃，使用 set_common_config_snippet）
pub fn set_claude_common_config_snippet(ctx: &CoreContext, snippet: &str) -> Result<(), String> {
    // 验证是否为有效的 JSON（如果不为空）
    if !snippet.trim().is_empty() {
        serde_json::from_str::<serde_json::Value>(snippet)
            .map_err(|e| format!("无效的 JSON 格式: {e}"))?;
    }

    let value = if snippet.trim().is_empty() {
        None
    } else {
        Some(snippet.to_string())
    };

    ctx.app_state()
        .db
        .set_config_snippet("claude", value)
        .map_err(|e| e.to_string())
}

/// 获取通用配置片段（统一接口）
pub fn get_common_config_snippet(ctx: &CoreContext, app_type: &str) -> Result<Option<String>, String> {
    ctx.app_state()
        .db
        .get_config_snippet(app_type)
        .map_err(|e| e.to_string())
}

/// 设置通用配置片段（统一接口）
pub fn set_common_config_snippet(
    ctx: &CoreContext,
    app_type: &str,
    snippet: &str,
) -> Result<(), String> {
    // 验证格式（根据应用类型）
    if !snippet.trim().is_empty() {
        match app_type {
            "claude" | "gemini" => {
                // 验证 JSON 格式
                serde_json::from_str::<serde_json::Value>(snippet)
                    .map_err(|e| format!("无效的 JSON 格式: {e}"))?;
            }
            "codex" => {
                // TOML 格式暂不验证
            }
            _ => {}
        }
    }

    let value = if snippet.trim().is_empty() {
        None
    } else {
        Some(snippet.to_string())
    };

    ctx.app_state()
        .db
        .set_config_snippet(app_type, value)
        .map_err(|e| e.to_string())
}

// ========================
// DeepLink 相关 API
// ========================

/// 导出 DeepLinkImportRequest 类型
pub use cc_switch::DeepLinkImportRequest;

/// 解析深链接 URL
pub fn parse_deeplink(url: &str) -> Result<cc_switch::DeepLinkImportRequest, String> {
    cc_switch::parse_deeplink_url(url).map_err(|e| e.to_string())
}

/// 合并深链接配置（从 Base64/URL 解析并填充完整配置）
pub fn merge_deeplink_config(
    request: cc_switch::DeepLinkImportRequest,
) -> Result<cc_switch::DeepLinkImportRequest, String> {
    cc_switch::parse_and_merge_config(&request).map_err(|e| e.to_string())
}

/// 统一导入深链接资源
pub fn import_from_deeplink_unified(
    ctx: &CoreContext,
    request: cc_switch::DeepLinkImportRequest,
) -> Result<serde_json::Value, String> {
    match request.resource.as_str() {
        "provider" => {
            let provider_id = cc_switch::import_provider_from_deeplink(ctx.app_state(), request)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "type": "provider",
                "id": provider_id
            }))
        }
        "prompt" => {
            let prompt_id = cc_switch::import_prompt_from_deeplink(ctx.app_state(), request)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "type": "prompt",
                "id": prompt_id
            }))
        }
        "mcp" => {
            let result = cc_switch::import_mcp_from_deeplink(ctx.app_state(), request)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "type": "mcp",
                "importedCount": result.imported_count,
                "importedIds": result.imported_ids,
                "failed": result.failed
            }))
        }
        "skill" => {
            let skill_key = cc_switch::import_skill_from_deeplink(ctx.app_state(), request)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::json!({
                "type": "skill",
                "key": skill_key
            }))
        }
        _ => Err(format!("Unsupported resource type: {}", request.resource)),
    }
}
