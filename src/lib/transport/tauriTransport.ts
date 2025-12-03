import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import type { ApiTransport, UnlistenFn } from "./types";

export const TauriTransport: ApiTransport = {
  mode: "tauri",

  async invoke<T = unknown>(command: string, payload?: unknown): Promise<T> {
    return tauriInvoke<T>(command, payload as Record<string, unknown>);
  },

  async listen<T = unknown>(
    event: string,
    handler: (payload: T) => void
  ): Promise<UnlistenFn> {
    const unlisten = await tauriListen<T>(event, (evt) => {
      handler(evt.payload);
    });
    return unlisten;
  },

  debug(msg: string, data?: unknown) {
    if (import.meta.env.DEV) {
      console.debug(`[TauriTransport] ${msg}`, data ?? "");
    }
  },
};
