import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
} from "react";
import { authApi } from "@/lib/api/auth";

interface AuthContextValue {
  isLoading: boolean;
  isAuthenticated: boolean;
  authEnabled: boolean;
  error: string | null;
  login: (password: string) => Promise<boolean>;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isLoading, setIsLoading] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [authEnabled, setAuthEnabled] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initAuth = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const statusRes = await authApi.checkStatus();
        setAuthEnabled(statusRes.enabled);

        if (statusRes.enabled) {
          const sessionRes = await authApi.checkSession();
          setIsAuthenticated(sessionRes.valid);
        } else {
          setIsAuthenticated(true);
        }
      } catch (err) {
        console.error("[Auth] Failed to initialize auth:", err);
        setError(err instanceof Error ? err.message : "Auth initialization failed");
        setAuthEnabled(false);
        setIsAuthenticated(true);
      } finally {
        setIsLoading(false);
      }
    };

    void initAuth();
  }, []);

  const login = useCallback(async (password: string): Promise<boolean> => {
    setError(null);

    try {
      const res = await authApi.login(password);

      if (res.success) {
        // Reload page to establish new WebSocket connection with auth cookie
        window.location.reload();
        return true;
      } else {
        setError(res.error || "Login failed");
        return false;
      }
    } catch (err) {
      console.error("[Auth] Login failed:", err);
      setError(err instanceof Error ? err.message : "Login failed");
      return false;
    }
  }, []);

  const value: AuthContextValue = {
    isLoading,
    isAuthenticated,
    authEnabled,
    error,
    login,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return context;
}
