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

const openFileDialogMock = vi.fn();
const importConfigMock = vi.fn();
const importUploadMock = vi.fn();
const saveFileDialogMock = vi.fn();
const exportConfigMock = vi.fn();

vi.mock("@/lib/api/settings", () => ({
  settingsApi: {
    openFileDialog: (...args: unknown[]) => openFileDialogMock(...args),
    importConfigFromFile: (...args: unknown[]) => importConfigMock(...args),
    importConfigFromUpload: (...args: unknown[]) => importUploadMock(...args),
    saveFileDialog: (...args: unknown[]) => saveFileDialogMock(...args),
    exportConfigToFile: (...args: unknown[]) => exportConfigMock(...args),
  },
  buildDefaultExportFileName: () => "cc-switch-export-test.sql",
}));

beforeEach(() => {
  openFileDialogMock.mockReset();
  importConfigMock.mockReset();
  importUploadMock.mockReset();
  saveFileDialogMock.mockReset();
  exportConfigMock.mockReset();
  toastSuccessMock.mockReset();
  toastErrorMock.mockReset();
  toastWarningMock.mockReset();
  Object.defineProperty(globalThis, "__TAURI__", {
    value: {},
    configurable: true,
  });
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("useImportExport Hook", () => {
  it("uses desktop mode by default in Tauri tests", () => {
    const { result } = renderHook(() => useImportExport());
    expect(result.current.importMode).toBe("desktop");
  });

  it("updates state after successfully selecting file", async () => {
    openFileDialogMock.mockResolvedValue("/path/config.sql");
    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    expect(result.current.selectedFile).toBe("/path/config.sql");
    expect(result.current.status).toBe("idle");
    expect(result.current.errorMessage).toBeNull();
  });

  it("shows error toast and keeps initial state when file dialog fails", async () => {
    openFileDialogMock.mockRejectedValue(new Error("file dialog error"));
    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    expect(result.current.selectedFile).toBe("");
    expect(result.current.status).toBe("idle");
  });

  it("shows error and returns early when no file is selected for import", async () => {
    const { result } = renderHook(() =>
      useImportExport({ onImportSuccess: vi.fn() }),
    );

    await act(async () => {
      await result.current.importConfig();
    });

    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    expect(importConfigMock).not.toHaveBeenCalled();
    expect(importUploadMock).not.toHaveBeenCalled();
    expect(result.current.status).toBe("idle");
  });

  it("sets success status, records backup ID, and calls callback on successful import", async () => {
    openFileDialogMock.mockResolvedValue("/config.sql");
    importConfigMock.mockResolvedValue({
      success: true,
      backupId: "backup-123",
    });
    const onImportSuccess = vi.fn();

    const { result } = renderHook(() => useImportExport({ onImportSuccess }));

    await act(async () => {
      await result.current.selectImportFile();
    });

    await act(async () => {
      await result.current.importConfig();
    });

    expect(importConfigMock).toHaveBeenCalledWith("/config.sql");
    expect(result.current.status).toBe("success");
    expect(result.current.backupId).toBe("backup-123");
    expect(toastSuccessMock).toHaveBeenCalledTimes(1);
    expect(onImportSuccess).toHaveBeenCalledTimes(1);
  });

  it("shows error message and keeps selected file when import result fails", async () => {
    openFileDialogMock.mockResolvedValue("/config.sql");
    importConfigMock.mockResolvedValue({
      success: false,
      message: "Config corrupted",
    });

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    await act(async () => {
      await result.current.importConfig();
    });

    expect(result.current.status).toBe("error");
    expect(result.current.errorMessage).toBe("Config corrupted");
    expect(result.current.selectedFile).toBe("/config.sql");
    expect(toastErrorMock).toHaveBeenCalledWith("Config corrupted");
  });

  it("shows partial-success when backend returns a post-import warning", async () => {
    openFileDialogMock.mockResolvedValue("/config.sql");
    importConfigMock.mockResolvedValue({
      success: true,
      backupId: "backup-123",
      warning: "Post-operation synchronization failed",
    });

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    await act(async () => {
      await result.current.importConfig();
    });

    expect(result.current.status).toBe("partial-success");
    expect(result.current.backupId).toBe("backup-123");
    expect(toastWarningMock).toHaveBeenCalledWith(
      "Post-operation synchronization failed",
      expect.objectContaining({ closeButton: true }),
    );
    expect(toastSuccessMock).not.toHaveBeenCalled();
  });

  it("catches and displays error when import process throws exception", async () => {
    openFileDialogMock.mockResolvedValue("/config.sql");
    importConfigMock.mockRejectedValue(new Error("Import failed"));

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    await act(async () => {
      await result.current.importConfig();
    });

    expect(result.current.status).toBe("error");
    expect(result.current.errorMessage).toBe("Import failed");
    expect(toastErrorMock).toHaveBeenCalledWith(
      expect.stringContaining("导入配置失败:"),
    );
  });

  it("exports successfully with default filename and shows path in toast", async () => {
    saveFileDialogMock.mockResolvedValue("/export.sql");
    exportConfigMock.mockResolvedValue({
      success: true,
      filePath: "/backup/export.sql",
    });

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.exportConfig();
    });

    expect(saveFileDialogMock).toHaveBeenCalledTimes(1);
    expect(exportConfigMock).toHaveBeenCalledWith("/export.sql");
    expect(toastSuccessMock).toHaveBeenCalledWith(
      expect.stringContaining("/backup/export.sql"),
      expect.objectContaining({ closeButton: true }),
    );
  });

  it("shows error message when export fails", async () => {
    saveFileDialogMock.mockResolvedValue("/export.sql");
    exportConfigMock.mockResolvedValue({
      success: false,
      message: "Write failed",
    });

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.exportConfig();
    });

    expect(toastErrorMock).toHaveBeenCalledWith(
      expect.stringContaining("Write failed"),
    );
  });

  it("catches and shows error when export throws exception", async () => {
    saveFileDialogMock.mockResolvedValue("/export.sql");
    exportConfigMock.mockRejectedValue(new Error("Disk read-only"));

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.exportConfig();
    });

    expect(toastErrorMock).toHaveBeenCalledWith(
      expect.stringContaining("Disk read-only"),
    );
  });

  it("shows error and returns when user cancels save dialog during export", async () => {
    saveFileDialogMock.mockResolvedValue(null);

    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.exportConfig();
    });

    expect(exportConfigMock).not.toHaveBeenCalled();
    expect(toastErrorMock).toHaveBeenCalledTimes(1);
  });

  it("restores initial values when clearing selection and resetting status", async () => {
    openFileDialogMock.mockResolvedValue("/config.sql");
    const { result } = renderHook(() => useImportExport());

    await act(async () => {
      await result.current.selectImportFile();
    });

    act(() => {
      result.current.clearSelection();
    });

    expect(result.current.selectedFile).toBe("");
    expect(result.current.status).toBe("idle");

    act(() => {
      result.current.resetStatus();
    });

    expect(result.current.errorMessage).toBeNull();
    expect(result.current.backupId).toBeNull();
  });
});
