// Auth API uses direct HTTP fetch to avoid WebSocket chicken-and-egg problem
// (WebSocket requires auth, but auth check needs to happen before WebSocket connects)

const API_BASE = import.meta.env.VITE_CC_SWITCH_API_BASE || "/api";

export interface AuthStatusResponse {
  enabled: boolean;
}

export interface LoginResponse {
  success: boolean;
  error?: string;
}

export interface SessionCheckResponse {
  valid: boolean;
}

async function authInvoke<T>(command: string, payload: unknown = {}): Promise<T> {
  const res = await fetch(`${API_BASE}/invoke`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include", // Include cookies
    body: JSON.stringify({ command, payload }),
  });

  const text = await res.text();
  if (!res.ok) {
    throw new Error(text || `Auth invoke failed for ${command}`);
  }

  const json = JSON.parse(text);
  if (json.error) {
    throw new Error(json.error);
  }
  return json.result as T;
}

export const authApi = {
  async checkStatus(): Promise<AuthStatusResponse> {
    return await authInvoke<AuthStatusResponse>("auth.status", {});
  },

  async login(password: string): Promise<LoginResponse> {
    return await authInvoke<LoginResponse>("auth.login", { password });
  },

  async checkSession(): Promise<SessionCheckResponse> {
    return await authInvoke<SessionCheckResponse>("auth.check", {});
  },
};
