import type { ReactNode } from "react";
import { act, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();
const listenMock = vi.fn();

vi.mock("@/App", async () => {
  const { useTranslation } = await import("react-i18next");

  return {
    default: function MockApp() {
      const { t } = useTranslation();

      return (
        <main>
          <h1>{t("errors.configLoadFailedTitle")}</h1>
          <p>
            {t("errors.configLoadFailedMessage", {
              path: "/tmp/config.json",
              detail: "boom",
            })}
          </p>
        </main>
      );
    },
  };
});

vi.mock("@/contexts/AuthContext", () => ({
  AuthProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

vi.mock("@/contexts/UpdateContext", () => ({
  UpdateProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

vi.mock("@/components/theme-provider", () => ({
  ThemeProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

vi.mock("@/components/ui/sonner", () => ({
  Toaster: () => null,
}));

vi.mock("@/lib/transport", () => ({
  invoke: invokeMock,
  listen: listenMock,
}));

vi.mock("@platform/bootstrap", async () => import("@/platform/bootstrap.web"));

describe("main web bootstrap", () => {
  beforeEach(() => {
    vi.resetModules();
    document.body.innerHTML = '<div id="root"></div>';
    document.body.className = "";
    localStorage.clear();
    localStorage.setItem("language", "zh");
    invokeMock.mockResolvedValue(null);
    listenMock.mockResolvedValue(() => {});
  });

  afterEach(() => {
    document.body.innerHTML = "";
    document.body.className = "";
    localStorage.clear();
    vi.clearAllMocks();
  });

  it("initializes i18n before rendering translation-bound UI", async () => {
    await act(async () => {
      await import("@/main");
    });

    await waitFor(() => {
      expect(screen.getByText("配置加载失败")).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_init_error");
    });

    const text = document.body.textContent ?? "";
    expect(text).toContain("/tmp/config.json");
    expect(text).toContain("boom");
    expect(text).not.toContain("errors.configLoadFailedTitle");
    expect(text).not.toContain("{{path}}");
    expect(text).not.toContain("{{detail}}");
    expect(listenMock).toHaveBeenCalledWith(
      "configLoadError",
      expect.any(Function),
    );
  });
});
