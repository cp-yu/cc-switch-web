import type { ApiTransport, UnlistenFn } from "./types";

const API_BASE = import.meta.env.VITE_CC_SWITCH_API_BASE || "/api";

async function httpInvoke<T>(command: string, payload?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}/invoke`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ command, payload: payload ?? {} }),
  });

  const text = await res.text();
  if (!res.ok) {
    throw new Error(text || `Invoke failed for ${command}`);
  }

  if (!text) return undefined as T;
  try {
    return JSON.parse(text) as T;
  } catch {
    return text as T;
  }
}

export const HttpTransport: ApiTransport = {
  mode: "http",

  invoke: httpInvoke,

  async listen<T = unknown>(
    _event: string,
    _handler: (payload: T) => void
  ): Promise<UnlistenFn> {
    console.warn("[HttpTransport] listen() not supported, returning no-op");
    return () => {};
  },

  debug(msg: string, data?: unknown) {
    if (import.meta.env.DEV) {
      console.debug(`[HttpTransport] ${msg}`, data ?? "");
    }
  },
};
