use serde_json::Value;
use crate::rpc::RpcError;

/// Dispatch a command to the appropriate handler
/// This is a placeholder - actual implementation will integrate with cc-switch-core
pub async fn dispatch_command(
    method: &str,
    params: &Value,
) -> Result<Value, RpcError> {
    // TODO: Integrate with cc-switch-core when available
    // For now, return a placeholder response
    match method {
        "ping" => Ok(serde_json::json!({ "pong": true })),

        // Provider commands (placeholder)
        "get_providers" => {
            let _app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");
            Ok(serde_json::json!({}))
        }

        "get_current_provider" => {
            let _app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");
            Ok(serde_json::json!(null))
        }

        // Settings commands (placeholder)
        "get_settings" => {
            Ok(serde_json::json!({
                "language": "en",
                "theme": "system"
            }))
        }

        // MCP commands (placeholder)
        "get_mcp_servers" => {
            Ok(serde_json::json!({}))
        }

        _ => Err(RpcError::method_not_found(method)),
    }
}
