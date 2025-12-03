export type TransportMode = "tauri" | "http" | "ws";

export type UnlistenFn = () => void | Promise<void>;

export interface ApiTransport {
  mode: TransportMode;
  invoke<T = unknown>(command: string, payload?: unknown): Promise<T>;
  listen<T = unknown>(
    event: string,
    handler: (payload: T) => void
  ): Promise<UnlistenFn>;
  debug?: (msg: string, data?: unknown) => void;
}
