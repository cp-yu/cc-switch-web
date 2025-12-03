import { TauriTransport } from "./tauriTransport";
import { HttpTransport } from "./httpTransport";
import { WebSocketTransport } from "./wsTransport";
import type { ApiTransport, UnlistenFn } from "./types";

export type { ApiTransport, UnlistenFn, TransportMode } from "./types";

let cachedTransport: ApiTransport | null = null;

function detectTransport(): ApiTransport {
  const mode = import.meta.env.VITE_CC_SWITCH_MODE;

  // Build-time forced mode
  if (mode === "ws" || mode === "websocket") {
    console.log("[Transport] Using WebSocket transport (build-time)");
    return WebSocketTransport;
  }
  if (mode === "http" || mode === "web") {
    console.log("[Transport] Using HTTP transport (build-time)");
    return HttpTransport;
  }
  if (mode === "tauri" || mode === "desktop") {
    console.log("[Transport] Using Tauri transport (build-time)");
    return TauriTransport;
  }

  // Runtime detection
  const isTauri =
    typeof window !== "undefined" && "__TAURI__" in (window as object);
  if (isTauri) {
    console.log("[Transport] Using Tauri transport (runtime detection)");
    return TauriTransport;
  }

  if (typeof WebSocket === "undefined") {
    console.warn("[Transport] WebSocket not supported, falling back to HTTP");
    return HttpTransport;
  }

  console.log("[Transport] Using WebSocket transport (runtime detection)");
  return WebSocketTransport;
}

export function getTransport(): ApiTransport {
  if (!cachedTransport) {
    cachedTransport = detectTransport();
  }
  return cachedTransport;
}

export function invoke<T = unknown>(
  command: string,
  payload?: unknown
): Promise<T> {
  return getTransport().invoke<T>(command, payload);
}

export function listen<T = unknown>(
  event: string,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  return getTransport().listen<T>(event, handler);
}

export function supportsRealtimeEvents(): boolean {
  const transport = getTransport();
  return transport.mode === "ws" || transport.mode === "tauri";
}

export function getTransportMode() {
  return getTransport().mode;
}

// For testing: allow injecting mock transport
export function __setTransportForTesting(t: ApiTransport | null) {
  cachedTransport = t;
}
