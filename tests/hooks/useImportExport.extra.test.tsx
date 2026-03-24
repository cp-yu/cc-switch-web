import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useImportExport } from "@/hooks/useImportExport";

const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();
const toastWarningMock = vi.fn();

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
    warning: (...args: unknown[]) => toastWarningMock(...args),
  },
}));

const importConfigMock = vi.fn();
const importUploadMock = vi.fn();
const saveFileDialogMock = vi.fn();
const exportConfigMock = vi.fn();
const exportDownloadMock = vi.fn();

vi.mock("@/lib/api/settings", () => ({
  settingsApi: {
    openFileDialog: vi.fn(),
    importConfigFromFile: (...args: unknown[]) => importConfigMock(...args),
    importConfigFromUpload: (...args: unknown[]) => importUploadMock(...args),
    saveFileDialog: (...args: unknown[]) => saveFileDialogMock(...args),
    exportConfigToFile: (...args: unknown[]) => exportConfigMock(...args),
    exportConfigForDownload: (...args: unknown[]) => exportDownloadMock(...args),
  },
  buildDefaultExportFileName: () => "cc-switch-export-test.sql",
}));

const originalTauri = (globalThis as Record<string, unknown>).__TAURI__;
const originalCreateObjectURL = URL.createObjectURL;
const originalRevokeObjectURL = URL.revokeObjectURL;

describe("useImportExport Hook (web mode)", () => {
  beforeEach(() => {
    importConfigMock.mockReset();
    importUploadMock.mockReset();
    saveFileDialogMock.mockReset();
    exportConfigMock.mockReset();
    exportDownloadMock.mockReset();
    toastSuccessMock.mockReset();
    toastErrorMock.mockReset();
    toastWarningMock.mockReset();
    delete (globalThis as Record<string, unknown>).__TAURI__;
    vi.useFakeTimers();
    URL.createObjectURL = vi.fn(() => "blob:mock-download");
    URL.revokeObjectURL = vi.fn();
  });

  afterEach(() => {
    if (originalTauri) {
      Object.defineProperty(globalThis, "__TAURI__", {
        value: originalTauri,
        configurable: true,
      });
    }
    URL.createObjectURL = originalCreateObjectURL;
    URL.revokeObjectURL = originalRevokeObjectURL;
    vi.useRealTimers();
  });

  it("switches to web mode and imports via upload API", async () => {
    importUploadMock.mockResolvedValue({
      success: true,
      backupId: "backup-web-1",
    });
    const file = new File(["-- CC Switch SQLite 导出\nSELECT 1;"], "config.sql", {
      type: "application/sql",
    });

    const { result } = renderHook(() => useImportExport());

    expect(result.current.importMode).toBe("web");

    act(() => {
      result.current.setUploadFile(file);
    });

    await act(async () => {
      await result.current.importConfig();
    });

    expect(importUploadMock).toHaveBeenCalledWith(file);
    expect(importConfigMock).not.toHaveBeenCalled();
    expect(result.current.selectedFile).toBe("config.sql");
    expect(result.current.backupId).toBe("backup-web-1");
  });

  it("resetStatus clears errors but preserves selected upload file", async () => {
    const file = new File(["broken"], "broken.sql", { type: "application/sql" });
    importUploadMock.mockResolvedValue({ success: false, message: "broken" });
    const { result } = renderHook(() => useImportExport());

    act(() => {
      result.current.setUploadFile(file);
    });

    await act(async () => {
      await result.current.importConfig();
    });

    act(() => {
      result.current.resetStatus();
    });

    expect(result.current.selectedFile).toBe("broken.sql");
    expect(result.current.status).toBe("idle");
    expect(result.current.errorMessage).toBeNull();
  });

  it("downloads exported sql in web mode without calling desktop APIs", async () => {
    exportDownloadMock.mockResolvedValue({
      blob: new Blob(["-- CC Switch SQLite 导出\nSELECT 1;"], {
        type: "application/sql",
      }),
      fileName: "cc-switch-export-20260324_120000.sql",
    });
    const clickSpy = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => {});

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.exportConfig();
    });

    expect(saveFileDialogMock).not.toHaveBeenCalled();
    expect(exportConfigMock).not.toHaveBeenCalled();
    expect(exportDownloadMock).toHaveBeenCalledTimes(1);
    expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    expect(clickSpy).toHaveBeenCalledTimes(1);
    expect(toastSuccessMock).toHaveBeenCalledWith(
      expect.stringContaining("cc-switch-export-20260324_120000.sql"),
      expect.objectContaining({ closeButton: true }),
    );

    clickSpy.mockRestore();
  });
});
