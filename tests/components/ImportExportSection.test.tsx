import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { ImportExportSection } from "@/components/settings/ImportExportSection";

const tMock = vi.fn((key: string) => key);

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

describe("ImportExportSection Component", () => {
  const baseProps = {
    importMode: "desktop" as const,
    status: "idle" as const,
    selectedFile: "",
    errorMessage: null,
    backupId: null,
    isImporting: false,
    isExporting: false,
    onSelectFile: vi.fn(),
    onSelectUploadFile: vi.fn(),
    onImport: vi.fn(),
    onExport: vi.fn(),
    onClear: vi.fn(),
  };

  beforeEach(() => {
    tMock.mockImplementation((key: string) => key);
    baseProps.onSelectFile.mockReset();
    baseProps.onSelectUploadFile.mockReset();
    baseProps.onImport.mockReset();
    baseProps.onExport.mockReset();
    baseProps.onClear.mockReset();
  });

  it("shows desktop import button and opens file dialog when no file is selected", () => {
    render(<ImportExportSection {...baseProps} />);

    expect(
      screen.getByRole("button", { name: /settings\.selectConfigFile/ }),
    ).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "settings.exportConfig" }),
    );
    expect(baseProps.onExport).toHaveBeenCalledTimes(1);

    fireEvent.click(
      screen.getByRole("button", { name: /settings\.selectConfigFile/ }),
    );
    expect(baseProps.onSelectFile).toHaveBeenCalledTimes(1);
  });

  it("shows filename and enables desktop import/clear when file is selected", () => {
    render(
      <ImportExportSection
        {...baseProps}
        selectedFile="/tmp/test/config.sql"
      />,
    );

    expect(screen.getByText(/config\.sql/)).toBeInTheDocument();
    const importButton = screen.getByRole("button", {
      name: /settings\.import/,
    });
    expect(importButton).toBeEnabled();
    fireEvent.click(importButton);
    expect(baseProps.onImport).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByRole("button", { name: "common.clear" }));
    expect(baseProps.onClear).toHaveBeenCalledTimes(1);
  });

  it("renders web upload controls and allows browser export", () => {
    render(
      <ImportExportSection
        {...baseProps}
        importMode="web"
        selectedFile="upload.sql"
      />,
    );

    expect(
      screen.getByRole("button", { name: "settings.import" }),
    ).toBeEnabled();
    const exportButton = screen.getByRole("button", {
      name: "settings.exportConfig",
    });
    expect(exportButton).toBeEnabled();
    fireEvent.click(exportButton);
    expect(baseProps.onExport).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/upload\.sql/)).toBeInTheDocument();
  });

  it("shows loading text and disables import button during import", () => {
    render(
      <ImportExportSection
        {...baseProps}
        selectedFile="/tmp/test/config.sql"
        isImporting
        status="importing"
      />,
    );

    const importingLabels = screen.getAllByText("settings.importing");
    expect(importingLabels.length).toBeGreaterThanOrEqual(2);
    expect(
      screen.getByRole("button", { name: "settings.importing" }),
    ).toBeDisabled();
    expect(screen.getByText("common.loading")).toBeInTheDocument();
  });

  it("shows exporting text and disables export button during export", () => {
    render(
      <ImportExportSection
        {...baseProps}
        isExporting
      />,
    );

    expect(
      screen.getByRole("button", { name: "settings.exporting" }),
    ).toBeDisabled();
  });

  it("displays backup information on successful import", () => {
    render(
      <ImportExportSection
        {...baseProps}
        selectedFile="/tmp/test/config.sql"
        status="success"
        backupId="backup-001"
      />,
    );

    expect(screen.getByText("settings.importSuccess")).toBeInTheDocument();
    expect(screen.getByText(/backup-001/)).toBeInTheDocument();
    expect(screen.getByText("settings.autoReload")).toBeInTheDocument();
  });

  it("displays error message when import fails", () => {
    render(
      <ImportExportSection
        {...baseProps}
        status="error"
        errorMessage="Parse failed"
      />,
    );

    expect(screen.getByText("settings.importFailed")).toBeInTheDocument();
    expect(screen.getByText("Parse failed")).toBeInTheDocument();
  });
});
