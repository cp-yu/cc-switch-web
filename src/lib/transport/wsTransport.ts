import type { ApiTransport, UnlistenFn } from "./types";

interface RpcRequest {
  jsonrpc: "2.0";
  id?: string;
  method: string;
  params?: unknown;
}

interface RpcError {
  code: number;
  message: string;
  data?: unknown;
}

interface RpcResponse {
  jsonrpc: "2.0";
  id?: string;
  result?: unknown;
  error?: RpcError;
  method?: string;
  params?: unknown;
}

type PendingRequest = {
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
};

class JsonRpcWebSocketClient {
  private socket: WebSocket | null = null;
  private nextId = 1;
  private pendingRequests = new Map<string, PendingRequest>();
  private subscriptions = new Map<string, Set<(payload: unknown) => void>>();
  private connectPromise: Promise<void> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private closedByUser = false;
  private reconnectDelay = 1000;
  private readonly maxReconnectDelay = 30000;

  constructor(private url: string) {}

  async connect(): Promise<void> {
    if (this.socket?.readyState === WebSocket.OPEN) return;
    if (this.connectPromise) return this.connectPromise;

    this.connectPromise = new Promise<void>((resolve, reject) => {
      const socket = new WebSocket(this.url);
      this.socket = socket;

      socket.onopen = () => {
        console.log("[WS] Connected");
        this.reconnectDelay = 1000;
        this.setupListeners(socket);
        resolve();
      };

      socket.onerror = (ev) => {
        console.error("[WS] Connection error", ev);
        reject(new Error("WebSocket connection failed"));
      };

      socket.onclose = () => {
        console.log("[WS] Connection closed");
        this.connectPromise = null;

        for (const [, pending] of this.pendingRequests) {
          pending.reject(new Error("Connection closed"));
        }
        this.pendingRequests.clear();

        if (!this.closedByUser) {
          this.scheduleReconnect();
        }
      };
    });

    return this.connectPromise;
  }

  private setupListeners(socket: WebSocket) {
    socket.onmessage = (ev) => {
      try {
        const msg: RpcResponse = JSON.parse(ev.data);

        if ("id" in msg && msg.id !== undefined) {
          this.handleResponse(msg);
          return;
        }

        if (msg.method === "event") {
          this.handleEvent(msg);
        }
      } catch (err) {
        console.error("[WS] Failed to parse message", err);
      }
    };
  }

  private handleResponse(response: RpcResponse) {
    const id = String(response.id);
    const pending = this.pendingRequests.get(id);
    if (!pending) return;

    this.pendingRequests.delete(id);

    if (response.error) {
      const err = new Error(response.error.message);
      (err as Error & { code?: number; data?: unknown }).code =
        response.error.code;
      (err as Error & { code?: number; data?: unknown }).data =
        response.error.data;
      pending.reject(err);
    } else {
      pending.resolve(response.result);
    }
  }

  private handleEvent(notification: RpcResponse) {
    const params = notification.params as { name: string; payload: unknown };
    if (!params?.name) return;

    const handlers = this.subscriptions.get(params.name);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(params.payload);
        } catch (err) {
          console.error(`[WS] Event handler error for ${params.name}`, err);
        }
      });
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;

    console.log(`[WS] Reconnecting in ${this.reconnectDelay}ms...`);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect()
        .then(() => this.resubscribeEvents())
        .catch(() => {
          this.reconnectDelay = Math.min(
            this.reconnectDelay * 2,
            this.maxReconnectDelay
          );
        });
    }, this.reconnectDelay);
  }

  private async resubscribeEvents() {
    const events = Array.from(this.subscriptions.keys());
    for (const event of events) {
      try {
        await this.sendRequest("event.subscribe", { event });
      } catch (err) {
        console.error(`[WS] Failed to resubscribe: ${event}`, err);
      }
    }
  }

  async sendRequest<T = unknown>(method: string, params?: unknown): Promise<T> {
    await this.connect();

    const id = String(this.nextId++);
    const request: RpcRequest = {
      jsonrpc: "2.0",
      id,
      method,
      params: params ?? {},
    };

    return new Promise<T>((resolve, reject) => {
      this.pendingRequests.set(id, {
        resolve: resolve as (v: unknown) => void,
        reject,
      });

      try {
        this.socket!.send(JSON.stringify(request));
      } catch (err) {
        this.pendingRequests.delete(id);
        reject(err);
      }
    });
  }

  async subscribe<T>(
    event: string,
    handler: (payload: T) => void
  ): Promise<UnlistenFn> {
    await this.connect();

    let handlers = this.subscriptions.get(event);
    const isFirst = !handlers;

    if (!handlers) {
      handlers = new Set();
      this.subscriptions.set(event, handlers);
    }

    handlers.add(handler as (payload: unknown) => void);

    if (isFirst) {
      try {
        await this.sendRequest("event.subscribe", { event });
      } catch (err) {
        handlers.delete(handler as (payload: unknown) => void);
        if (handlers.size === 0) {
          this.subscriptions.delete(event);
        }
        throw err;
      }
    }

    return async () => {
      const handlers = this.subscriptions.get(event);
      if (!handlers) return;

      handlers.delete(handler as (payload: unknown) => void);

      if (handlers.size === 0) {
        this.subscriptions.delete(event);
        try {
          await this.sendRequest("event.unsubscribe", { event });
        } catch (err) {
          console.error(`[WS] Failed to unsubscribe: ${event}`, err);
        }
      }
    };
  }

  close() {
    this.closedByUser = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.socket?.close();
  }
}

function buildWsUrl(): string {
  const apiBase = import.meta.env.VITE_CC_SWITCH_API_BASE || "/api";
  const { protocol, host } = window.location;
  const wsProtocol = protocol === "https:" ? "wss:" : "ws:";
  const url = new URL(`${wsProtocol}//${host}${apiBase}/ws`);

  const token =
    import.meta.env.VITE_CC_SWITCH_AUTH_TOKEN ||
    localStorage.getItem("cc_switch_auth_token");
  if (token) {
    url.searchParams.set("auth", token);
  }

  return url.toString();
}

let wsClient: JsonRpcWebSocketClient | null = null;

function getWsClient(): JsonRpcWebSocketClient {
  if (!wsClient) {
    wsClient = new JsonRpcWebSocketClient(buildWsUrl());
  }
  return wsClient;
}

export const WebSocketTransport: ApiTransport = {
  mode: "ws",

  async invoke<T = unknown>(command: string, payload?: unknown): Promise<T> {
    return getWsClient().sendRequest<T>(command, payload);
  },

  async listen<T = unknown>(
    event: string,
    handler: (payload: T) => void
  ): Promise<UnlistenFn> {
    return getWsClient().subscribe<T>(event, handler);
  },

  debug(msg: string, data?: unknown) {
    if (import.meta.env.DEV) {
      console.debug(`[WebSocketTransport] ${msg}`, data ?? "");
    }
  },
};

if (typeof window !== "undefined") {
  window.addEventListener("beforeunload", () => {
    wsClient?.close();
  });
}
