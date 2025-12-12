use std::sync::Arc;

use serde_json::Value;

use crate::{auth::verify_password, rpc::RpcError, ServerState};

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
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let providers =
                cc_switch_core::get_providers(core, app).map_err(RpcError::app_error)?;

            serde_json::to_value(providers).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_current_provider" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id =
                cc_switch_core::get_current_provider(core, app).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(id))
        }

        "add_provider" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_value = params
                .get("provider")
                .ok_or_else(|| RpcError::invalid_params("missing 'provider' field"))?;

            let provider: cc_switch_core::CoreProvider =
                serde_json::from_value(provider_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'provider' value: {e}"))
                })?;

            let ok =
                cc_switch_core::add_provider(core, app, provider).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "update_provider" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_value = params
                .get("provider")
                .ok_or_else(|| RpcError::invalid_params("missing 'provider' field"))?;

            let provider: cc_switch_core::CoreProvider =
                serde_json::from_value(provider_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'provider' value: {e}"))
                })?;

            let ok =
                cc_switch_core::update_provider(core, app, provider).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "delete_provider" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let ok =
                cc_switch_core::delete_provider(core, app, id).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "switch_provider" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let ok =
                cc_switch_core::switch_provider(core, app, id).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "import_default_config" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let ok = cc_switch_core::import_default_config(core, app)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        // Web / server 模式下托盘菜单更新为 no-op，只返回 true
        "update_tray_menu" => {
            let ok =
                cc_switch_core::update_tray_menu(core).map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "update_providers_sort_order" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .unwrap_or("claude");

            let updates = params
                .get("updates")
                .ok_or_else(|| RpcError::invalid_params("missing 'updates' field"))?;

            let ok = cc_switch_core::update_providers_sort_order(core, app, updates)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "queryProviderUsage" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let usage =
                cc_switch_core::query_provider_usage(core, app, provider_id)
                    .await
                    .map_err(RpcError::app_error)?;

            Ok(usage)
        }

        "testUsageScript" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let script_code = params
                .get("scriptCode")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'scriptCode' field"))?;

            let timeout = params.get("timeout").and_then(|v| v.as_u64());
            let api_key = params.get("apiKey").and_then(|v| v.as_str());
            let base_url = params.get("baseUrl").and_then(|v| v.as_str());
            let access_token = params.get("accessToken").and_then(|v| v.as_str());
            let user_id = params.get("userId").and_then(|v| v.as_str());

            let result = cc_switch_core::test_usage_script(
                core,
                app,
                provider_id,
                script_code,
                timeout,
                api_key,
                base_url,
                access_token,
                user_id,
            )
            .await
            .map_err(RpcError::app_error)?;

            Ok(result)
        }

        "read_live_provider_settings" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let settings =
                cc_switch_core::read_live_provider_settings(app)
                    .map_err(RpcError::app_error)?;

            Ok(settings)
        }

        "test_api_endpoints" => {
            let urls_value = params
                .get("urls")
                .ok_or_else(|| RpcError::invalid_params("missing 'urls' field"))?;

            let urls: Vec<String> = serde_json::from_value(urls_value.clone()).map_err(|e| {
                RpcError::invalid_params(format!("invalid 'urls' value: {e}"))
            })?;

            let timeout_secs = params.get("timeoutSecs").and_then(|v| v.as_u64());

            let result = cc_switch_core::test_api_endpoints(urls, timeout_secs)
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_custom_endpoints" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let endpoints =
                cc_switch_core::get_custom_endpoints(core, app, provider_id)
                    .map_err(RpcError::app_error)?;

            Ok(endpoints)
        }

        "add_custom_endpoint" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'url' field"))?;

            cc_switch_core::add_custom_endpoint(core, app, provider_id, url.to_string())
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "remove_custom_endpoint" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'url' field"))?;

            cc_switch_core::remove_custom_endpoint(core, app, provider_id, url.to_string())
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "update_endpoint_last_used" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let provider_id = params
                .get("providerId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'providerId' field"))?;

            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'url' field"))?;

            cc_switch_core::update_endpoint_last_used(core, app, provider_id, url.to_string())
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        // Settings commands
        "get_settings" => {
            let settings = cc_switch_core::get_settings();
            serde_json::to_value(settings)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "save_settings" => {
            let settings_value = params
                .get("settings")
                .ok_or_else(|| RpcError::invalid_params("missing 'settings' field"))?;

            let settings: cc_switch_core::CoreAppSettings =
                serde_json::from_value(settings_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'settings' value: {e}"))
                })?;

            let ok = cc_switch_core::save_settings(settings).map_err(RpcError::app_error)?;
            Ok(serde_json::json!(ok))
        }

        "restart_app" => {
            // Stub for web server - not applicable
            let ok = cc_switch_core::restart_app().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(ok))
        }

        "check_for_updates" => {
            // Returns the update URL for client to handle
            let url = cc_switch_core::check_for_updates().map_err(RpcError::app_error)?;
            Ok(serde_json::json!({ "url": url }))
        }

        "is_portable_mode" => {
            let is_portable = cc_switch_core::is_portable_mode().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(is_portable))
        }

        "get_config_dir" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");
            let dir = cc_switch_core::get_config_dir(app).map_err(RpcError::app_error)?;
            Ok(serde_json::json!(dir))
        }

        "open_config_folder" => {
            // Stub for web server - returns path for client to handle
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");
            let path = cc_switch_core::open_config_folder(app).map_err(RpcError::app_error)?;
            Ok(serde_json::json!({ "path": path }))
        }

        "pick_directory" => {
            // Stub for web server - not applicable
            let result = cc_switch_core::pick_directory().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(result))
        }

        "get_claude_code_config_path" => {
            let path = cc_switch_core::get_claude_code_config_path();
            Ok(serde_json::json!(path))
        }

        "get_app_config_path" => {
            let path = cc_switch_core::get_app_config_path();
            Ok(serde_json::json!(path))
        }

        "open_app_config_folder" => {
            // Stub for web server - returns path for client to handle
            let path = cc_switch_core::open_app_config_folder().map_err(RpcError::app_error)?;
            Ok(serde_json::json!({ "path": path }))
        }

        "get_app_config_dir_override" => {
            let override_path = cc_switch_core::get_app_config_dir_override();
            Ok(serde_json::json!(override_path))
        }

        "set_app_config_dir_override" => {
            let path = params.get("path").and_then(|v| v.as_str());
            let ok = cc_switch_core::set_app_config_dir_override(path)
                .map_err(RpcError::app_error)?;
            Ok(serde_json::json!(ok))
        }

        "apply_claude_plugin_config" => {
            let official = params
                .get("official")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let ok = cc_switch_core::apply_claude_plugin_config(official)
                .map_err(RpcError::app_error)?;
            Ok(serde_json::json!(ok))
        }

        "save_file_dialog" => {
            // Stub for web server - not applicable
            let result = cc_switch_core::save_file_dialog().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(result))
        }

        "open_file_dialog" => {
            // Stub for web server - not applicable
            let result = cc_switch_core::open_file_dialog().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(result))
        }

        "export_config_to_file" => {
            let file_path = params
                .get("filePath")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'filePath' field"))?;

            let result = cc_switch_core::export_config_to_file(core, file_path)
                .map_err(RpcError::app_error)?;
            Ok(result)
        }

        "import_config_from_file" => {
            let file_path = params
                .get("filePath")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'filePath' field"))?;

            let result = cc_switch_core::import_config_from_file(core, file_path)
                .map_err(RpcError::app_error)?;
            Ok(result)
        }

        "sync_current_providers_live" => {
            let result = cc_switch_core::sync_current_providers_live(core)
                .map_err(RpcError::app_error)?;
            Ok(result)
        }

        "open_external" => {
            // Stub for web server - returns URL for client to handle
            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'url' field"))?;

            let resolved_url = cc_switch_core::open_external(url).map_err(RpcError::app_error)?;
            Ok(serde_json::json!({ "url": resolved_url }))
        }

        "set_auto_launch" => {
            // Stub for web server - not applicable
            let enabled = params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let ok = cc_switch_core::set_auto_launch(enabled).map_err(RpcError::app_error)?;
            Ok(serde_json::json!(ok))
        }

        "get_auto_launch_status" => {
            // Stub for web server - returns false
            let status = cc_switch_core::get_auto_launch_status().map_err(RpcError::app_error)?;
            Ok(serde_json::json!(status))
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

        "get_claude_mcp_status" => {
            let status =
                cc_switch_core::get_claude_mcp_status().map_err(RpcError::app_error)?;

            serde_json::to_value(status)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "read_claude_mcp_config" => {
            let content =
                cc_switch_core::read_claude_mcp_config().map_err(RpcError::app_error)?;

            Ok(serde_json::json!(content))
        }

        "upsert_claude_mcp_server" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let spec = params
                .get("spec")
                .cloned()
                .ok_or_else(|| RpcError::invalid_params("missing 'spec' field"))?;

            let ok = cc_switch_core::upsert_claude_mcp_server(id, spec)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "delete_claude_mcp_server" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let ok = cc_switch_core::delete_claude_mcp_server(id)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "validate_mcp_command" => {
            let cmd = params
                .get("cmd")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'cmd' field"))?;

            let ok = cc_switch_core::validate_mcp_command(cmd)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "get_mcp_config" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let config =
                cc_switch_core::get_mcp_config(core, app).map_err(RpcError::app_error)?;

            serde_json::to_value(config)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "upsert_mcp_server_in_config" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let spec = params
                .get("spec")
                .cloned()
                .ok_or_else(|| RpcError::invalid_params("missing 'spec' field"))?;

            let sync_other_side = params.get("syncOtherSide").and_then(|v| v.as_bool());

            let ok = cc_switch_core::upsert_mcp_server_in_config(core, app, id, spec, sync_other_side)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "delete_mcp_server_in_config" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let ok = cc_switch_core::delete_mcp_server_in_config(core, id)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "set_mcp_enabled" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let enabled = params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| RpcError::invalid_params("missing 'enabled' field"))?;

            let ok = cc_switch_core::set_mcp_enabled(core, app, id, enabled)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "upsert_mcp_server" => {
            let server_value = params
                .get("server")
                .ok_or_else(|| RpcError::invalid_params("missing 'server' field"))?;

            let server: cc_switch_core::CoreMcpServer =
                serde_json::from_value(server_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'server' value: {e}"))
                })?;

            cc_switch_core::upsert_mcp_server(core, server)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "delete_mcp_server" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let ok = cc_switch_core::delete_mcp_server(core, id)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(ok))
        }

        "toggle_mcp_app" => {
            let server_id = params
                .get("serverId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'serverId' field"))?;

            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'app' field"))?;

            let enabled = params
                .get("enabled")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| RpcError::invalid_params("missing 'enabled' field"))?;

            cc_switch_core::toggle_mcp_app(core, server_id, app, enabled)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        // ========================
        // Prompt commands
        // ========================

        "get_prompts" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let prompts = cc_switch_core::get_prompts(core, app)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(prompts)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "upsert_prompt" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            let prompt_value = params
                .get("prompt")
                .ok_or_else(|| RpcError::invalid_params("missing 'prompt' field"))?;

            let prompt: cc_switch_core::Prompt =
                serde_json::from_value(prompt_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'prompt' value: {e}"))
                })?;

            cc_switch_core::upsert_prompt(core, app, id, prompt)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "delete_prompt" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            cc_switch_core::delete_prompt(core, app, id)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "enable_prompt" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'id' field"))?;

            cc_switch_core::enable_prompt(core, app, id)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "import_prompt_from_file" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let id = cc_switch_core::import_prompt_from_file(core, app)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(id))
        }

        "get_current_prompt_file_content" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("claude");

            let content = cc_switch_core::get_current_prompt_file_content(app)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(content))
        }

        // ========================
        // Environment commands
        // ========================

        "check_env_conflicts" => {
            let app = params
                .get("app")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'app' field"))?;

            let conflicts = cc_switch_core::check_env_conflicts(app)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(conflicts)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "delete_env_vars" => {
            let conflicts_value = params
                .get("conflicts")
                .ok_or_else(|| RpcError::invalid_params("missing 'conflicts' field"))?;

            let conflicts: Vec<cc_switch_core::EnvConflict> =
                serde_json::from_value(conflicts_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'conflicts' value: {e}"))
                })?;

            let backup_info = cc_switch_core::delete_env_vars(conflicts)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(backup_info)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "restore_env_backup" => {
            let backup_path = params
                .get("backupPath")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'backupPath' field"))?;

            cc_switch_core::restore_env_backup(backup_path)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        // ========================
        // Config snippet commands
        // ========================

        "get_claude_common_config_snippet" => {
            let snippet = cc_switch_core::get_claude_common_config_snippet(core)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(snippet))
        }

        "set_claude_common_config_snippet" => {
            let snippet = params
                .get("snippet")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            cc_switch_core::set_claude_common_config_snippet(core, snippet)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_common_config_snippet" => {
            let app_type = params
                .get("appType")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'appType' field"))?;

            let snippet = cc_switch_core::get_common_config_snippet(core, app_type)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(snippet))
        }

        "set_common_config_snippet" => {
            let app_type = params
                .get("appType")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'appType' field"))?;

            let snippet = params
                .get("snippet")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            cc_switch_core::set_common_config_snippet(core, app_type, snippet)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        // ========================
        // DeepLink commands
        // ========================

        "parse_deeplink" => {
            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'url' field"))?;

            let request = cc_switch_core::parse_deeplink(url)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(request)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "merge_deeplink_config" => {
            let request_value = params
                .get("request")
                .ok_or_else(|| RpcError::invalid_params("missing 'request' field"))?;

            let request: cc_switch_core::DeepLinkImportRequest =
                serde_json::from_value(request_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'request' value: {e}"))
                })?;

            let merged = cc_switch_core::merge_deeplink_config(request)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(merged)
                .map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "import_from_deeplink_unified" => {
            let request_value = params
                .get("request")
                .ok_or_else(|| RpcError::invalid_params("missing 'request' field"))?;

            let request: cc_switch_core::DeepLinkImportRequest =
                serde_json::from_value(request_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'request' value: {e}"))
                })?;

            let result = cc_switch_core::import_from_deeplink_unified(core, request)
                .map_err(RpcError::app_error)?;

            Ok(result)
        }

        // Misc commands
        "get_init_error" => {
            // Web 服务器环境下没有初始化错误
            Ok(serde_json::json!(null))
        }

        "get_migration_result" => {
            // Web 服务器环境下没有迁移
            Ok(serde_json::json!(false))
        }

        // ========================
        // Auth commands
        // ========================

        "auth.status" => {
            let enabled = state.auth_config.is_some();
            Ok(serde_json::json!({ "enabled": enabled }))
        }

        "auth.login" => {
            // Check if auth is enabled
            let auth_config = match &state.auth_config {
                Some(config) => config,
                None => {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "Authentication not configured"
                    }));
                }
            };

            // Get password from params
            let password = params
                .get("password")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'password' field"))?;

            // Verify password
            if !verify_password(password, &auth_config.password_hash) {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "Invalid password"
                }));
            }

            // Create session
            let token = state.session_store.create_session();

            Ok(serde_json::json!({
                "success": true,
                "token": token
            }))
        }

        "auth.check" => {
            let token = params
                .get("token")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError::invalid_params("missing 'token' field"))?;

            let valid = state.session_store.validate_session(token);
            Ok(serde_json::json!({ "valid": valid }))
        }

        _ => Err(RpcError::method_not_found(method)),
    }
}
