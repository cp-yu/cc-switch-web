use std::sync::Arc;

use serde_json::Value;

use crate::{auth::verify_password, events::ServerEvent, rpc::RpcError, ServerState};

macro_rules! rpc_business_methods {
    ($($method:literal),+ $(,)?) => {
        pub const RPC_BUSINESS_METHODS: &[&str] = &[$($method),+];
    };
}

rpc_business_methods!(
    "get_providers",
    "get_current_provider",
    "add_provider",
    "update_provider",
    "delete_provider",
    "switch_provider",
    "import_default_config",
    "update_tray_menu",
    "update_providers_sort_order",
    "queryProviderUsage",
    "testUsageScript",
    "read_live_provider_settings",
    "test_api_endpoints",
    "get_custom_endpoints",
    "add_custom_endpoint",
    "remove_custom_endpoint",
    "update_endpoint_last_used",
    "stream_check_provider",
    "stream_check_all_providers",
    "get_stream_check_config",
    "save_stream_check_config",
    "get_usage_summary",
    "get_usage_trends",
    "get_provider_stats",
    "get_model_stats",
    "get_request_logs",
    "get_request_detail",
    "get_model_pricing",
    "update_model_pricing",
    "delete_model_pricing",
    "check_provider_limits",
    "start_proxy_server",
    "stop_proxy_with_restore",
    "get_proxy_takeover_status",
    "set_proxy_takeover_for_app",
    "get_proxy_status",
    "get_proxy_config",
    "update_proxy_config",
    "get_global_proxy_config",
    "update_global_proxy_config",
    "get_proxy_config_for_app",
    "update_proxy_config_for_app",
    "is_proxy_running",
    "is_live_takeover_active",
    "switch_proxy_provider",
    "get_provider_health",
    "reset_circuit_breaker",
    "get_circuit_breaker_config",
    "update_circuit_breaker_config",
    "get_circuit_breaker_stats",
    "get_failover_queue",
    "get_available_providers_for_failover",
    "add_to_failover_queue",
    "remove_from_failover_queue",
    "get_auto_failover_enabled",
    "set_auto_failover_enabled",
    "get_default_cost_multiplier",
    "set_default_cost_multiplier",
    "get_pricing_model_source",
    "set_pricing_model_source",
    "get_settings",
    "save_settings",
    "restart_app",
    "check_for_updates",
    "is_portable_mode",
    "get_config_dir",
    "open_config_folder",
    "pick_directory",
    "get_claude_code_config_path",
    "get_app_config_path",
    "open_app_config_folder",
    "get_app_config_dir_override",
    "set_app_config_dir_override",
    "apply_claude_plugin_config",
    "save_file_dialog",
    "open_file_dialog",
    "export_config_to_file",
    "import_config_from_file",
    "sync_current_providers_live",
    "open_external",
    "set_auto_launch",
    "get_auto_launch_status",
    "get_skills",
    "get_mcp_servers",
    "get_claude_mcp_status",
    "read_claude_mcp_config",
    "upsert_claude_mcp_server",
    "delete_claude_mcp_server",
    "validate_mcp_command",
    "get_mcp_config",
    "upsert_mcp_server_in_config",
    "delete_mcp_server_in_config",
    "set_mcp_enabled",
    "upsert_mcp_server",
    "delete_mcp_server",
    "toggle_mcp_app",
    "get_prompts",
    "upsert_prompt",
    "delete_prompt",
    "enable_prompt",
    "import_prompt_from_file",
    "get_current_prompt_file_content",
    "check_env_conflicts",
    "delete_env_vars",
    "restore_env_backup",
    "get_claude_common_config_snippet",
    "set_claude_common_config_snippet",
    "get_common_config_snippet",
    "set_common_config_snippet",
    "parse_deeplink",
    "merge_deeplink_config",
    "import_from_deeplink_unified",
    "get_init_error",
    "get_migration_result",
);

fn get_str_param<'a>(params: &'a Value, keys: &[&str]) -> Result<&'a str, RpcError> {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_str()))
        .ok_or_else(|| RpcError::invalid_params(format!("missing '{}' field", keys[0])))
}

fn get_bool_param(params: &Value, keys: &[&str]) -> Result<bool, RpcError> {
    keys.iter()
        .find_map(|key| params.get(*key).and_then(|v| v.as_bool()))
        .ok_or_else(|| RpcError::invalid_params(format!("missing '{}' field", keys[0])))
}

fn get_optional_i64_param(params: &Value, keys: &[&str]) -> Result<Option<i64>, RpcError> {
    for key in keys {
        match params.get(*key) {
            Some(value) => {
                return value
                    .as_i64()
                    .map(Some)
                    .ok_or_else(|| RpcError::invalid_params(format!("invalid '{}' field", key)));
            }
            None => continue,
        }
    }

    Ok(None)
}

fn get_optional_u32_param(params: &Value, keys: &[&str]) -> Result<Option<u32>, RpcError> {
    for key in keys {
        match params.get(*key) {
            Some(value) => {
                let raw = value
                    .as_u64()
                    .ok_or_else(|| RpcError::invalid_params(format!("invalid '{}' field", key)))?;
                let parsed = u32::try_from(raw)
                    .map_err(|_| RpcError::invalid_params(format!("invalid '{}' field", key)))?;
                return Ok(Some(parsed));
            }
            None => continue,
        }
    }

    Ok(None)
}

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
            let template_type = params.get("templateType").and_then(|v| v.as_str());

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
                template_type,
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

        "stream_check_provider" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;

            let result = cc_switch_core::stream_check_provider(core, app_type, provider_id)
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "stream_check_all_providers" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let proxy_targets_only = params
                .get("proxyTargetsOnly")
                .or_else(|| params.get("proxy_targets_only"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let result = cc_switch_core::stream_check_all_providers(core, app_type, proxy_targets_only)
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(result).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_stream_check_config" => {
            let config = cc_switch_core::get_stream_check_config(core).map_err(RpcError::app_error)?;

            serde_json::to_value(config).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "save_stream_check_config" => {
            let config_value = params
                .get("config")
                .ok_or_else(|| RpcError::invalid_params("missing 'config' field"))?;

            let config: cc_switch_core::StreamCheckConfig =
                serde_json::from_value(config_value.clone()).map_err(|e| {
                    RpcError::invalid_params(format!("invalid 'config' value: {e}"))
                })?;

            cc_switch_core::save_stream_check_config(core, config).map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_usage_summary" => {
            let start_date = get_optional_i64_param(params, &["startDate", "start_date"])?;
            let end_date = get_optional_i64_param(params, &["endDate", "end_date"])?;

            let summary = cc_switch_core::get_usage_summary(core, start_date, end_date)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(summary).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_usage_trends" => {
            let start_date = get_optional_i64_param(params, &["startDate", "start_date"])?;
            let end_date = get_optional_i64_param(params, &["endDate", "end_date"])?;

            let trends = cc_switch_core::get_usage_trends(core, start_date, end_date)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(trends).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_provider_stats" => {
            let stats = cc_switch_core::get_provider_stats(core).map_err(RpcError::app_error)?;

            serde_json::to_value(stats).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_model_stats" => {
            let stats = cc_switch_core::get_model_stats(core).map_err(RpcError::app_error)?;

            serde_json::to_value(stats).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_request_logs" => {
            let filters_value = params
                .get("filters")
                .ok_or_else(|| RpcError::invalid_params("missing 'filters' field"))?;
            let filters: cc_switch_core::LogFilters = serde_json::from_value(filters_value.clone())
                .map_err(|e| RpcError::invalid_params(format!("invalid 'filters' value: {e}")))?;
            let page = get_optional_u32_param(params, &["page"])?.unwrap_or(0);
            let page_size = get_optional_u32_param(params, &["pageSize", "page_size"])?
                .unwrap_or(20);

            let logs = cc_switch_core::get_request_logs(core, filters, page, page_size)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(logs).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_request_detail" => {
            let request_id = get_str_param(params, &["requestId", "request_id"])?;

            let detail = cc_switch_core::get_request_detail(core, request_id)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(detail).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_model_pricing" => {
            let pricing = cc_switch_core::get_model_pricing(core).map_err(RpcError::app_error)?;

            serde_json::to_value(pricing).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "update_model_pricing" => {
            let model_id = get_str_param(params, &["modelId", "model_id"])?;
            let display_name = get_str_param(params, &["displayName", "display_name"])?;
            let input_cost = get_str_param(params, &["inputCost", "input_cost"])?;
            let output_cost = get_str_param(params, &["outputCost", "output_cost"])?;
            let cache_read_cost = get_str_param(params, &["cacheReadCost", "cache_read_cost"])?;
            let cache_creation_cost =
                get_str_param(params, &["cacheCreationCost", "cache_creation_cost"])?;

            cc_switch_core::update_model_pricing(
                core,
                model_id.to_string(),
                display_name.to_string(),
                input_cost.to_string(),
                output_cost.to_string(),
                cache_read_cost.to_string(),
                cache_creation_cost.to_string(),
            )
            .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "delete_model_pricing" => {
            let model_id = get_str_param(params, &["modelId", "model_id"])?;

            cc_switch_core::delete_model_pricing(core, model_id.to_string())
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "check_provider_limits" => {
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let status = cc_switch_core::check_provider_limits(core, provider_id, app_type)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(status).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        // Proxy commands
        "start_proxy_server" => {
            let info = core
                .app_state()
                .proxy_service
                .start()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(info).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "stop_proxy_with_restore" => {
            core.app_state()
                .proxy_service
                .stop_with_restore()
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_proxy_takeover_status" => {
            let status = core
                .app_state()
                .proxy_service
                .get_takeover_status()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(status).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "set_proxy_takeover_for_app" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let enabled = get_bool_param(params, &["enabled"])?;

            core.app_state()
                .proxy_service
                .set_takeover_for_app(app_type, enabled)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_proxy_status" => {
            let status = core
                .app_state()
                .proxy_service
                .get_status()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(status).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_proxy_config" => {
            let config = core
                .app_state()
                .proxy_service
                .get_config()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(config).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "update_proxy_config" => {
            let config_value = params
                .get("config")
                .ok_or_else(|| RpcError::invalid_params("missing 'config' field"))?;

            let config = serde_json::from_value(config_value.clone())
                .map_err(|e| RpcError::invalid_params(format!("invalid 'config' value: {e}")))?;

            core.app_state()
                .proxy_service
                .update_config(&config)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_global_proxy_config" => {
            let config = core
                .app_state()
                .db
                .get_global_proxy_config()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(config).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "update_global_proxy_config" => {
            let config_value = params
                .get("config")
                .ok_or_else(|| RpcError::invalid_params("missing 'config' field"))?;

            let config = serde_json::from_value(config_value.clone())
                .map_err(|e| RpcError::invalid_params(format!("invalid 'config' value: {e}")))?;

            core.app_state()
                .db
                .update_global_proxy_config(config)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_proxy_config_for_app" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let config = core
                .app_state()
                .db
                .get_proxy_config_for_app(app_type)
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(config).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "update_proxy_config_for_app" => {
            let config_value = params
                .get("config")
                .ok_or_else(|| RpcError::invalid_params("missing 'config' field"))?;

            let config = serde_json::from_value(config_value.clone())
                .map_err(|e| RpcError::invalid_params(format!("invalid 'config' value: {e}")))?;

            core.app_state()
                .db
                .update_proxy_config_for_app(config)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "is_proxy_running" => Ok(serde_json::json!(core.app_state().proxy_service.is_running().await)),

        "is_live_takeover_active" => {
            let active = core
                .app_state()
                .proxy_service
                .is_takeover_active()
                .await
                .map_err(RpcError::app_error)?;
            Ok(serde_json::json!(active))
        }

        "switch_proxy_provider" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;

            core.app_state()
                .proxy_service
                .switch_proxy_target(app_type, provider_id)
                .await
                .map_err(RpcError::app_error)?;

            let _ = state.event_bus.send(ServerEvent {
                name: "provider-switched".to_string(),
                payload: serde_json::json!({
                    "appType": app_type,
                    "providerId": provider_id,
                    "source": "webProxySwitch"
                }),
            });

            Ok(Value::Null)
        }

        "get_provider_health" => {
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let health = core
                .app_state()
                .db
                .get_provider_health(provider_id, app_type)
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(health).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "reset_circuit_breaker" => {
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            core.app_state()
                .db
                .update_provider_health(provider_id, app_type, true, None)
                .await
                .map_err(RpcError::app_error)?;

            core.app_state()
                .proxy_service
                .reset_provider_circuit_breaker(provider_id, app_type)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_circuit_breaker_config" => {
            let config = core
                .app_state()
                .db
                .get_circuit_breaker_config()
                .await
                .map_err(RpcError::app_error)?;

            serde_json::to_value(config).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "update_circuit_breaker_config" => {
            let config_value = params
                .get("config")
                .ok_or_else(|| RpcError::invalid_params("missing 'config' field"))?;

            let config = serde_json::from_value(config_value.clone())
                .map_err(|e| RpcError::invalid_params(format!("invalid 'config' value: {e}")))?;

            core.app_state()
                .db
                .update_circuit_breaker_config(&config)
                .await
                .map_err(RpcError::app_error)?;

            core.app_state()
                .proxy_service
                .update_circuit_breaker_configs(config)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_circuit_breaker_stats" => Ok(Value::Null),

        "get_failover_queue" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let queue = core
                .app_state()
                .db
                .get_failover_queue(app_type)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(queue).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "get_available_providers_for_failover" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let providers = core
                .app_state()
                .db
                .get_available_providers_for_failover(app_type)
                .map_err(RpcError::app_error)?;

            serde_json::to_value(providers).map_err(|e| RpcError::internal_error(e.to_string()))
        }

        "add_to_failover_queue" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;

            core.app_state()
                .db
                .add_to_failover_queue(app_type, provider_id)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "remove_from_failover_queue" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let provider_id = get_str_param(params, &["providerId", "provider_id"])?;

            core.app_state()
                .db
                .remove_from_failover_queue(app_type, provider_id)
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_auto_failover_enabled" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let enabled = core
                .app_state()
                .db
                .get_proxy_config_for_app(app_type)
                .await
                .map(|config| config.auto_failover_enabled)
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(enabled))
        }

        "set_auto_failover_enabled" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let enabled = get_bool_param(params, &["enabled"])?;

            let p1_provider_id = if enabled {
                let mut queue = core
                    .app_state()
                    .db
                    .get_failover_queue(app_type)
                    .map_err(RpcError::app_error)?;

                if queue.is_empty() {
                    let current_id = core
                        .app_state()
                        .db
                        .get_current_provider(app_type)
                        .map_err(RpcError::app_error)?
                        .ok_or_else(|| {
                            RpcError::app_error(
                                "故障转移队列为空，且未设置当前供应商，无法开启故障转移",
                            )
                        })?;

                    core.app_state()
                        .db
                        .add_to_failover_queue(app_type, &current_id)
                        .map_err(RpcError::app_error)?;

                    queue = core
                        .app_state()
                        .db
                        .get_failover_queue(app_type)
                        .map_err(RpcError::app_error)?;
                }

                Some(
                    queue
                        .first()
                        .map(|item| item.provider_id.clone())
                        .ok_or_else(|| RpcError::app_error("故障转移队列为空，无法开启故障转移"))?,
                )
            } else {
                None
            };

            let mut config = core
                .app_state()
                .db
                .get_proxy_config_for_app(app_type)
                .await
                .map_err(RpcError::app_error)?;
            config.auto_failover_enabled = enabled;

            core.app_state()
                .db
                .update_proxy_config_for_app(config)
                .await
                .map_err(RpcError::app_error)?;

            if let Some(provider_id) = p1_provider_id {
                core.app_state()
                    .proxy_service
                    .switch_proxy_target(app_type, &provider_id)
                    .await
                    .map_err(RpcError::app_error)?;

                let _ = state.event_bus.send(ServerEvent {
                    name: "provider-switched".to_string(),
                    payload: serde_json::json!({
                        "appType": app_type,
                        "providerId": provider_id,
                        "source": "failoverEnabled"
                    }),
                });
            }

            Ok(Value::Null)
        }

        "get_default_cost_multiplier" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let value = core
                .app_state()
                .db
                .get_default_cost_multiplier(app_type)
                .await
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(value))
        }

        "set_default_cost_multiplier" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let value = get_str_param(params, &["value"])?;

            core.app_state()
                .db
                .set_default_cost_multiplier(app_type, value)
                .await
                .map_err(RpcError::app_error)?;

            Ok(Value::Null)
        }

        "get_pricing_model_source" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;

            let value = core
                .app_state()
                .db
                .get_pricing_model_source(app_type)
                .await
                .map_err(RpcError::app_error)?;

            Ok(serde_json::json!(value))
        }

        "set_pricing_model_source" => {
            let app_type = get_str_param(params, &["appType", "app_type"])?;
            let value = get_str_param(params, &["value"])?;

            core.app_state()
                .db
                .set_pricing_model_source(app_type, value)
                .await
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
