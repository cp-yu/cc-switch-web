//! cc-switch-core
//!
//! 该 crate 提供与 UI 无关的核心业务封装，供 Web 服务器等复用。
//! 当前实现基于现有的 `cc_switch`（src-tauri）进行轻量封装，
//! 后续可以逐步将纯业务逻辑下沉到本 crate。

use std::sync::Arc;
use std::str::FromStr;

use cc_switch::{
    AppError, AppSettings, AppState, AppType, Database, McpServer, Provider, ProviderService,
    SkillService,
};
use indexmap::IndexMap;

/// 对外暴露的核心类型别名，便于直接使用
pub use cc_switch::{
    AppSettings as CoreAppSettings, AppType as CoreAppType, McpServer as CoreMcpServer,
    Provider as CoreProvider,
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

        let skill_service = SkillService::new().map(Arc::new).ok();

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
pub fn get_providers(
    ctx: &CoreContext,
    app: &str,
) -> Result<IndexMap<String, Provider>, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::list(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

/// 获取指定应用的当前供应商 ID
pub fn get_current_provider(ctx: &CoreContext, app: &str) -> Result<String, String> {
    let app_type = AppType::from_str(app).map_err(|e| e.to_string())?;
    ProviderService::current(ctx.app_state(), app_type).map_err(|e| e.to_string())
}

// ========================
// Settings 相关 API
// ========================

/// 获取应用设置（通过现有 Tauri 命令，保证行为与桌面端一致）
pub async fn get_settings() -> Result<AppSettings, String> {
    cc_switch::get_settings().await
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
        .list_skills(repos)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(skills).map_err(|e| e.to_string())
}

// ========================
// MCP 相关 API
// ========================

/// 获取所有 MCP 服务器（统一结构）
pub fn get_mcp_servers(
    ctx: &CoreContext,
) -> Result<IndexMap<String, McpServer>, String> {
    cc_switch::McpService::get_all_servers(ctx.app_state())
        .map_err(|e| e.to_string())
}
