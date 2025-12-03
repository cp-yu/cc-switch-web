use std::sync::Arc;

use serde_json::Value;

use crate::{rpc::RpcError, ServerState};

/// Dispatch a command to the appropriate handler
pub async fn dispatch_command(
    state: &Arc<ServerState>,
    method: &str,
    params: &Value,
) -> Result<Value, RpcError> {
    let core = &state.core;

    match method {
        "ping" => Ok(serde_json::json!({ "pong": true })),

        // Provider commands
        "get_providers" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");

            let providers =
                cc_switch_core::get_providers(core, app).map_err(RpcError::app_error)?;

            serde_json::to_value(providers)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_current_provider" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");

            let id =
                cc_switch_core::get_current_provider(core, app).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(id))
        }

        // Settings commands
        "get_settings" => {
            let settings = cc_switch_core::get_settings()
                .await
                .map_err(RpcError::app_error)?;
            serde_json::to_value(settings)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        // Skill commands
        "get_skills" => {
            let skills = cc_switch_core::get_skills(core)
                .await
                .map_err(RpcError::app_error)?;

            Ok(skills)
        }

        // MCP commands
        "get_mcp_servers" => {
            let servers =
                cc_switch_core::get_mcp_servers(core).map_err(RpcError::app_error)?;

            serde_json::to_value(servers)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        _ => Err(RpcError::method_not_found(method)),
    }
}
