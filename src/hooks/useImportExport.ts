import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { settingsApi, buildDefaultExportFileName } from "@/lib/api/settings";

export type ImportStatus =
  | "idle"
  | "importing"
  | "success"
  | "partial-success"
  | "error";

export type ImportMode = "desktop" | "web";

export interface UseImportExportOptions {
  onImportSuccess?: () => void | Promise<void>;
}

export interface UseImportExportResult {
  importMode: ImportMode;
  selectedFile: string;
  status: ImportStatus;
  errorMessage: string | null;
  backupId: string | null;
  isImporting: boolean;
  isExporting: boolean;
  selectImportFile: () => Promise<void>;
  setUploadFile: (file: File | null) => void;
  clearSelection: () => void;
  importConfig: () => Promise<void>;
  exportConfig: () => Promise<void>;
  resetStatus: () => void;
}

function isDesktopRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in (window as object);
}

function triggerBrowserDownload(blob: Blob, fileName: string) {
  const downloadUrl = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = downloadUrl;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(downloadUrl), 0);
}

export function useImportExport(
  options: UseImportExportOptions = {},
): UseImportExportResult {
  const { t } = useTranslation();
  const { onImportSuccess } = options;

  const importMode: ImportMode = useMemo(
    () => (isDesktopRuntime() ? "desktop" : "web"),
    [],
  );
  const [selectedFilePath, setSelectedFilePath] = useState("");
  const [selectedUploadFile, setSelectedUploadFile] = useState<File | null>(null);
  const [status, setStatus] = useState<ImportStatus>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [backupId, setBackupId] = useState<string | null>(null);
  const [isImporting, setIsImporting] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  const selectedFile =
    importMode === "desktop"
      ? selectedFilePath
      : (selectedUploadFile?.name ?? "");

  const clearSelection = useCallback(() => {
    setSelectedFilePath("");
    setSelectedUploadFile(null);
    setStatus("idle");
    setErrorMessage(null);
    setBackupId(null);
  }, []);

  const setUploadFile = useCallback((file: File | null) => {
    setSelectedUploadFile(file);
    setStatus("idle");
    setErrorMessage(null);
    setBackupId(null);
  }, []);

  const selectImportFile = useCallback(async () => {
    if (importMode !== "desktop") {
      return;
    }

    try {
      const filePath = await settingsApi.openFileDialog();
      if (filePath) {
        setSelectedFilePath(filePath);
        setStatus("idle");
        setErrorMessage(null);
      }
    } catch (error) {
      console.error("[useImportExport] Failed to open file dialog", error);
      toast.error(
        t("settings.selectFileFailed", {
          defaultValue: "选择文件失败",
        }),
      );
    }
  }, [importMode, t]);

  const importConfig = useCallback(async () => {
    if (importMode === "desktop" && !selectedFilePath) {
      toast.error(
        t("settings.selectFileFailed", {
          defaultValue: "请选择有效的 SQL 备份文件",
        }),
      );
      return;
    }

    if (importMode === "web" && !selectedUploadFile) {
      toast.error(
        t("settings.selectFileFailed", {
          defaultValue: "请选择有效的 SQL 备份文件",
        }),
      );
      return;
    }

    if (isImporting || isExporting) return;

    setIsImporting(true);
    setStatus("importing");
    setErrorMessage(null);

    try {
      const result =
        importMode === "desktop"
          ? await settingsApi.importConfigFromFile(selectedFilePath)
          : await settingsApi.importConfigFromUpload(selectedUploadFile!);
      if (!result.success) {
        setStatus("error");
        const message =
          result.message ||
          t("settings.configCorrupted", {
            defaultValue: "SQL 文件已损坏或格式不正确",
          });
        setErrorMessage(message);
        toast.error(message);
        return;
      }

      setBackupId(result.backupId ?? null);
      void onImportSuccess?.();

      if (result.warning) {
        setStatus("partial-success");
        toast.warning(result.warning, { closeButton: true });
        return;
      }

      setStatus("success");
      toast.success(
        t("settings.importSuccess", {
          defaultValue: "配置导入成功",
        }),
        { closeButton: true },
      );
    } catch (error) {
      console.error("[useImportExport] Failed to import config", error);
      setStatus("error");
      const message =
        error instanceof Error ? error.message : String(error ?? "");
      setErrorMessage(message);
      toast.error(
        t("settings.importFailedError", {
          defaultValue: "导入配置失败: {{message}}",
          message,
        }),
      );
    } finally {
      setIsImporting(false);
    }
  }, [
    importMode,
    isExporting,
    isImporting,
    onImportSuccess,
    selectedFilePath,
    selectedUploadFile,
    t,
  ]);

  const exportConfig = useCallback(async () => {
    if (isImporting || isExporting) return;

    setIsExporting(true);
    try {
      if (importMode === "web") {
        const { blob, fileName } = await settingsApi.exportConfigForDownload();
        triggerBrowserDownload(blob, fileName);
        toast.success(
          t("settings.configExported", {
            defaultValue: "配置已导出",
          }) + `
${fileName}`,
          { closeButton: true },
        );
        return;
      }

      const defaultName = buildDefaultExportFileName();
      const destination = await settingsApi.saveFileDialog(defaultName);
      if (!destination) {
        toast.error(
          t("settings.selectFileFailed", {
            defaultValue: "请选择 SQL 备份保存路径",
          }),
        );
        return;
      }

      const result = await settingsApi.exportConfigToFile(destination);
      if (result.success) {
        const displayPath = result.filePath ?? destination;
        toast.success(
          t("settings.configExported", {
            defaultValue: "配置已导出",
          }) + `
${displayPath}`,
          { closeButton: true },
        );
      } else {
        toast.error(
          t("settings.exportFailed", {
            defaultValue: "导出配置失败",
          }) + (result.message ? `: ${result.message}` : ""),
        );
      }
    } catch (error) {
      console.error("[useImportExport] Failed to export config", error);
      toast.error(
        t("settings.exportFailedError", {
          defaultValue: "导出配置失败: {{message}}",
          message: error instanceof Error ? error.message : String(error ?? ""),
        }),
      );
    } finally {
      setIsExporting(false);
    }
  }, [importMode, isExporting, isImporting, t]);

  const resetStatus = useCallback(() => {
    setStatus("idle");
    setErrorMessage(null);
    setBackupId(null);
  }, []);

  return {
    importMode,
    selectedFile,
    status,
    errorMessage,
    backupId,
    isImporting,
    isExporting,
    selectImportFile,
    setUploadFile,
    clearSelection,
    importConfig,
    exportConfig,
    resetStatus,
  };
}
