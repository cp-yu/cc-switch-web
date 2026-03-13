use std::collections::BTreeSet;

use cc_switch_core::WEB_COMPAT_TAURI_COMMANDS;
use cc_switch_server::api::{PUBLIC_METHODS, RPC_BUSINESS_METHODS, WS_PROTOCOL_METHODS};

fn sorted_set<'a>(items: &'a [&'a str]) -> BTreeSet<&'a str> {
    items.iter().copied().collect()
}

#[test]
fn tauri_and_rpc_business_methods_stay_in_sync() {
    let tauri_methods = sorted_set(WEB_COMPAT_TAURI_COMMANDS);
    let rpc_methods = sorted_set(RPC_BUSINESS_METHODS);

    let missing_in_rpc: Vec<_> = tauri_methods.difference(&rpc_methods).copied().collect();
    assert!(
        missing_in_rpc.is_empty(),
        "web-compatible Tauri commands missing in RPC dispatch: {:?}",
        missing_in_rpc
    );

    let unexpected_rpc: Vec<_> = rpc_methods.difference(&tauri_methods).copied().collect();
    assert!(
        unexpected_rpc.is_empty(),
        "RPC business methods missing in Tauri compatibility list: {:?}",
        unexpected_rpc
    );
}

#[test]
fn protocol_method_whitelists_only_reference_live_entries() {
    let rpc_methods = sorted_set(RPC_BUSINESS_METHODS);
    let tauri_methods = sorted_set(WEB_COMPAT_TAURI_COMMANDS);
    let invoke_public = sorted_set(PUBLIC_METHODS);
    let ws_protocol = sorted_set(WS_PROTOCOL_METHODS);

    for method in PUBLIC_METHODS {
        assert!(
            !rpc_methods.contains(method),
            "public auth method {method} should stay out of RPC business methods"
        );
        assert!(
            !tauri_methods.contains(method),
            "public auth method {method} should stay out of Tauri compatibility methods"
        );
        assert!(
            invoke_public.contains(method),
            "public auth method whitelist contains stale entry {method}"
        );
    }

    for method in WS_PROTOCOL_METHODS {
        assert!(
            !rpc_methods.contains(method),
            "WS protocol method {method} should stay out of RPC business methods"
        );
        assert!(
            !tauri_methods.contains(method),
            "WS protocol method {method} should stay out of Tauri compatibility methods"
        );
        assert!(
            ws_protocol.contains(method),
            "WS protocol whitelist contains stale entry {method}"
        );
    }
}
