/**
 * Import Wizard — multi-step dialog for importing bank statement files.
 *
 * Steps: Upload → Configure → Preview → Confirm
 */

import {
  createSignal,
  createResource,
  For,
  Show,
  Switch as SolidSwitch,
  Match,
  type Component,
} from "solid-js";
import {
  Upload,
  FileSpreadsheet,
  CheckCircle2,
  AlertCircle,
  ChevronRight,
  X,
} from "lucide-solid";
import {
  Button,
  Dialog,
  Select,
  showToast,
} from "~/components/ui";
import {
  api,
  type Account,
  type Bank,
  type UploadResponse,
  type Import,
  type ImportExecutionResult,
} from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";

// ── Types ────────────────────────────────────────────────────

type WizardStep = "upload" | "configure" | "preview" | "confirm";

interface ImportWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: () => void;
}

// ── Data fetching ────────────────────────────────────────────

async function fetchBanksAndAccounts(): Promise<{ banks: Bank[]; accounts: Account[] }> {
  const [banksRes, accountsRes] = await Promise.all([
    api.fetchList<Bank>("/api/banks"),
    api.fetchList<Account>("/api/accounts"),
  ]);
  return { banks: banksRes.data, accounts: accountsRes.data };
}

// ── Component ────────────────────────────────────────────────

export const ImportWizard: Component<ImportWizardProps> = (props) => {
  const t = useI18n();

  // Step state
  const [step, setStep] = createSignal<WizardStep>("upload");

  // Upload state
  const [selectedFile, setSelectedFile] = createSignal<File | null>(null);
  const [accountId, setAccountId] = createSignal("");
  const [uploading, setUploading] = createSignal(false);
  const [dragOver, setDragOver] = createSignal(false);

  // Upload response
  const [uploadResult, setUploadResult] = createSignal<UploadResponse | null>(null);

  // Configure state (column mapping)
  const [mapping, setMapping] = createSignal<Record<string, string>>({});

  // Execute state
  const [executing, setExecuting] = createSignal(false);
  const [executionResult, setExecutionResult] = createSignal<ImportExecutionResult | null>(null);

  // Data
  const [data] = createResource(fetchBanksAndAccounts);

  const accountOptions = () => {
    const d = data();
    if (!d) return [];
    return d.accounts.map((a) => {
      const bank = d.banks.find((b) => b.id === a.bank_id);
      return {
        value: a.id,
        label: `${bank?.name ?? "Unknown"} — ${a.name} (${a.currency})`,
      };
    });
  };

  // ── Reset ──────────────────────────────────────────────────

  const resetWizard = () => {
    setStep("upload");
    setSelectedFile(null);
    setAccountId("");
    setUploading(false);
    setDragOver(false);
    setUploadResult(null);
    setMapping({});
    setExecuting(false);
    setExecutionResult(null);
  };

  const handleClose = () => {
    props.onOpenChange(false);
    // Reset after close animation
    setTimeout(resetWizard, 200);
  };

  // ── Step 1: Upload ────────────────────────────────────────

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    if (input.files?.[0]) {
      setSelectedFile(input.files[0]);
    }
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    if (e.dataTransfer?.files?.[0]) {
      setSelectedFile(e.dataTransfer.files[0]);
    }
  };

  const handleUpload = async () => {
    const file = selectedFile();
    const acctId = accountId();
    if (!file || !acctId) return;

    setUploading(true);
    try {
      const formData = new FormData();
      formData.append("file", file);
      formData.append("account_id", acctId);

      const res = await api.postFormData<{ data: UploadResponse }>(
        "/api/imports/upload",
        formData,
      );
      setUploadResult(res.data);
      showToast({ title: t("import.toast.uploaded") ?? "File uploaded", variant: "success" });

      // If CSV/JSON, go to configure; otherwise skip to preview
      const fmt = res.data.detected_format.toLowerCase();
      if (fmt === "csv" || fmt === "json") {
        setStep("configure");
      } else {
        setStep("preview");
      }
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Upload failed.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setUploading(false);
    }
  };

  // ── Step 2: Configure mapping ─────────────────────────────

  const handleSaveMapping = async () => {
    const result = uploadResult();
    if (!result) return;

    try {
      await api.updateOne<Import>(
        `/api/imports/${result.import.id}/configure`,
        { mapping: mapping() },
      );
      showToast({ title: t("import.toast.configured") ?? "Mapping saved", variant: "success" });
      setStep("preview");
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to save mapping.";
      showToast({ title: msg, variant: "error" });
    }
  };

  // ── Step 3: Preview → Execute ─────────────────────────────

  const handleExecute = async () => {
    const result = uploadResult();
    if (!result) return;

    setExecuting(true);
    try {
      const res = await api.post<{ data: ImportExecutionResult }>(
        `/api/imports/${result.import.id}/execute`,
        {
          mapping: Object.keys(mapping()).length > 0 ? mapping() : undefined,
          skip_duplicates: true,
        },
      );
      setExecutionResult(res.data);
      showToast({ title: t("import.toast.imported") ?? "Import completed", variant: "success" });
      setStep("confirm");
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Import failed.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setExecuting(false);
    }
  };

  // ── Step indicators ───────────────────────────────────────

  const steps: WizardStep[] = ["upload", "configure", "preview", "confirm"];
  const stepIndex = () => steps.indexOf(step());

  return (
    <Dialog
      open={props.open}
      onOpenChange={(open) => { if (!open) handleClose(); }}
      title={t("import.wizard.title") ?? "Import Transactions"}
    >
      <div class="space-y-4 pt-2 w-full">
        {/* Step indicator */}
        <div class="flex items-center justify-center gap-1 text-xs">
          <For each={steps}>
            {(s, i) => (
              <>
                <Show when={i() > 0}>
                  <ChevronRight size={12} class="text-text-tertiary" />
                </Show>
                <span
                  class={`px-2 py-0.5 rounded-full transition-colors ${
                    step() === s
                      ? "bg-primary text-white font-medium"
                      : i() < stepIndex()
                        ? "text-primary font-medium"
                        : "text-text-tertiary"
                  }`}
                >
                  {t(`import.wizard.steps.${s}`) ?? s}
                </span>
              </>
            )}
          </For>
        </div>

        {/* Step content */}
        <SolidSwitch>
          {/* ── Upload ─────────────────────────────────────── */}
          <Match when={step() === "upload"}>
            <div class="space-y-4">
              {/* Account select */}
              <Select
                name="account"
                label={t("import.upload.selectAccount") ?? "Select account"}
                options={accountOptions()}
                value={accountId()}
                onChange={setAccountId}
                placeholder={t("import.upload.selectAccount") ?? "Select account"}
                required
              />

              {/* Drag & drop zone */}
              <div
                class={`relative flex flex-col items-center justify-center gap-3 rounded-[var(--radius-lg)] border-2 border-dashed p-8 transition-colors cursor-pointer ${
                  dragOver()
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-primary/50"
                }`}
                onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
                onDragLeave={() => setDragOver(false)}
                onDrop={handleDrop}
                onClick={() => document.getElementById("import-file-input")?.click()}
              >
                <Upload size={32} class="text-text-tertiary" />
                <Show
                  when={selectedFile()}
                  fallback={
                    <>
                      <p class="text-sm font-medium text-text">
                        {t("import.upload.description") ?? "Drag and drop a file, or click to browse"}
                      </p>
                      <p class="text-xs text-text-tertiary">
                        {t("import.upload.supportedFormats") ?? "CSV, MT940, OFX, QIF, CAMT.053, XLSX, JSON, PDF"}
                      </p>
                    </>
                  }
                >
                  <div class="flex items-center gap-2">
                    <FileSpreadsheet size={16} class="text-primary" />
                    <span class="text-sm font-medium text-text">{selectedFile()!.name}</span>
                    <button
                      class="text-text-tertiary hover:text-text cursor-pointer"
                      onClick={(e) => { e.stopPropagation(); setSelectedFile(null); }}
                    >
                      <X size={14} />
                    </button>
                  </div>
                </Show>
                <input
                  id="import-file-input"
                  type="file"
                  class="hidden"
                  accept=".csv,.mt940,.sta,.ofx,.qfx,.qif,.xml,.xlsx,.xls,.ods,.json,.pdf"
                  onChange={handleFileSelect}
                />
              </div>

              {/* Actions */}
              <div class="flex justify-end gap-2">
                <Button variant="secondary" size="sm" onClick={handleClose}>
                  {t("common.actions.cancel") ?? "Cancel"}
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  onClick={handleUpload}
                  loading={uploading()}
                  disabled={!selectedFile() || !accountId()}
                >
                  {t("common.actions.next") ?? "Next"}
                </Button>
              </div>
            </div>
          </Match>

          {/* ── Configure ──────────────────────────────────── */}
          <Match when={step() === "configure"}>
            <div class="space-y-4">
              <p class="text-sm text-text-secondary">
                {t("import.configure.description") ?? "Map your file's columns to transaction fields."}
              </p>

              {/* Column mapping UI */}
              <Show when={uploadResult()}>
                {(result) => {
                  const preview = result().preview;
                  const firstRow = preview[0];
                  if (!firstRow) return null;

                  const targetFields = [
                    { value: "skip", label: t("import.configure.skip") ?? "Skip" },
                    { value: "date", label: t("import.configure.date") ?? "Date" },
                    { value: "amount", label: t("import.configure.amount") ?? "Amount" },
                    { value: "description", label: t("import.configure.description") ?? "Description" },
                    { value: "payee", label: t("import.configure.payee") ?? "Payee" },
                    { value: "reference", label: t("import.configure.reference") ?? "Reference" },
                    { value: "currency", label: t("import.configure.currency") ?? "Currency" },
                  ];

                  // Extract column names from metadata keys
                  const columnKeys = Object.keys(firstRow.metadata);

                  return (
                    <div class="space-y-3">
                      <For each={columnKeys}>
                        {(col) => (
                          <div class="flex items-center gap-3">
                            <span class="text-sm font-medium text-text w-32 truncate">{col}</span>
                            <span class="text-text-tertiary">→</span>
                            <Select
                              name={`map-${col}`}
                              label=""
                              options={targetFields}
                              value={mapping()[col] ?? "skip"}
                              onChange={(v) => setMapping((m) => ({ ...m, [col]: v }))}
                            />
                            <span class="text-xs text-text-tertiary truncate max-w-[120px]">
                              {String(firstRow.metadata[col] ?? "")}
                            </span>
                          </div>
                        )}
                      </For>
                    </div>
                  );
                }}
              </Show>

              {/* Actions */}
              <div class="flex justify-between">
                <Button variant="secondary" size="sm" onClick={() => setStep("upload")}>
                  {t("common.actions.back") ?? "Back"}
                </Button>
                <Button variant="primary" size="sm" onClick={handleSaveMapping}>
                  {t("common.actions.next") ?? "Next"}
                </Button>
              </div>
            </div>
          </Match>

          {/* ── Preview ────────────────────────────────────── */}
          <Match when={step() === "preview"}>
            <div class="space-y-4">
              <Show when={uploadResult()}>
                {(result) => (
                  <>
                    {/* Stats bar */}
                    <div class="flex items-center gap-4 text-sm">
                      <span class="text-text font-medium">
                        {t("import.upload.formatDetected") ?? "Format"}: {result().detected_format}
                      </span>
                      <span class="text-text-secondary">
                        {result().total_rows} {t("import.preview.rows") ?? "rows"}
                      </span>
                    </div>

                    {/* Preview table */}
                    <div class="overflow-x-auto rounded-[var(--radius-md)] border border-border">
                      <table class="w-full text-sm">
                        <thead>
                          <tr class="bg-surface border-b border-border">
                            <th class="px-3 py-2 text-left text-text-secondary font-medium">#</th>
                            <th class="px-3 py-2 text-left text-text-secondary font-medium">
                              {t("transactions.columns.date") ?? "Date"}
                            </th>
                            <th class="px-3 py-2 text-left text-text-secondary font-medium">
                              {t("transactions.columns.description") ?? "Description"}
                            </th>
                            <th class="px-3 py-2 text-right text-text-secondary font-medium">
                              {t("transactions.columns.amount") ?? "Amount"}
                            </th>
                            <th class="px-3 py-2 text-left text-text-secondary font-medium">
                              {t("transactions.columns.payee") ?? "Payee"}
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          <For each={result().preview.slice(0, 20)}>
                            {(row, i) => (
                              <tr class="border-b border-border last:border-b-0 hover:bg-surface-hover transition-colors">
                                <td class="px-3 py-2 text-text-tertiary">{i() + 1}</td>
                                <td class="px-3 py-2 text-text">{row.date}</td>
                                <td class="px-3 py-2 text-text truncate max-w-[200px]">{row.description}</td>
                                <td class={`px-3 py-2 text-right font-medium tabular-nums ${
                                  parseFloat(row.amount) >= 0 ? "text-income" : "text-expense"
                                }`}>
                                  {row.currency ?? ""} {row.amount}
                                </td>
                                <td class="px-3 py-2 text-text-secondary truncate max-w-[150px]">
                                  {row.payee ?? "—"}
                                </td>
                              </tr>
                            )}
                          </For>
                        </tbody>
                      </table>
                    </div>

                    <Show when={result().preview.length > 20}>
                      <p class="text-xs text-text-tertiary text-center">
                        Showing 20 of {result().total_rows} rows
                      </p>
                    </Show>
                  </>
                )}
              </Show>

              {/* Actions */}
              <div class="flex justify-between">
                <Button variant="secondary" size="sm" onClick={() => {
                  const fmt = uploadResult()?.detected_format.toLowerCase();
                  setStep(fmt === "csv" || fmt === "json" ? "configure" : "upload");
                }}>
                  {t("common.actions.back") ?? "Back"}
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  onClick={handleExecute}
                  loading={executing()}
                >
                  {t("common.actions.confirm") ?? "Confirm Import"}
                </Button>
              </div>
            </div>
          </Match>

          {/* ── Confirm (Result) ───────────────────────────── */}
          <Match when={step() === "confirm"}>
            <div class="space-y-4">
              <div class="flex flex-col items-center gap-3 py-4">
                <CheckCircle2 size={48} class="text-income" />
                <h3 class="text-lg font-semibold text-text">
                  {t("import.confirm.title") ?? "Import Complete"}
                </h3>
              </div>

              <Show when={executionResult()}>
                {(result) => (
                  <div class="rounded-[var(--radius-md)] border border-border p-4 space-y-2">
                    <SummaryRow
                      label={t("import.confirm.imported") ?? "Imported"}
                      value={result().imported_count}
                      variant="success"
                    />
                    <SummaryRow
                      label={t("import.confirm.duplicates") ?? "Duplicates"}
                      value={result().duplicate_count}
                      variant={result().duplicate_count > 0 ? "warning" : "neutral"}
                    />
                    <SummaryRow
                      label={t("import.confirm.errors") ?? "Errors"}
                      value={result().error_count}
                      variant={result().error_count > 0 ? "danger" : "neutral"}
                    />

                    {/* Error details */}
                    <Show when={result().errors.length > 0}>
                      <div class="mt-3 space-y-1">
                        <For each={result().errors.slice(0, 10)}>
                          {(err) => (
                            <div class="flex items-start gap-2 text-xs">
                              <AlertCircle size={12} class="text-danger mt-0.5 shrink-0" />
                              <span class="text-text-secondary">
                                Row {err.row}: {err.message}
                              </span>
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                )}
              </Show>

              {/* Actions */}
              <div class="flex justify-end gap-2">
                <Button variant="secondary" size="sm" onClick={() => {
                  resetWizard();
                }}>
                  {t("import.confirm.importAnother") ?? "Import Another"}
                </Button>
                <Button variant="primary" size="sm" onClick={() => {
                  handleClose();
                  props.onComplete?.();
                }}>
                  {t("import.confirm.done") ?? "Done"}
                </Button>
              </div>
            </div>
          </Match>
        </SolidSwitch>
      </div>
    </Dialog>
  );
};

// ── Summary row helper ──────────────────────────────────────

function SummaryRow(props: {
  label: string;
  value: number;
  variant: "success" | "warning" | "danger" | "neutral";
}) {
  const colorClass = () => {
    switch (props.variant) {
      case "success": return "text-income";
      case "warning": return "text-warning";
      case "danger": return "text-danger";
      default: return "text-text-secondary";
    }
  };

  return (
    <div class="flex items-center justify-between text-sm">
      <span class="text-text-secondary">{props.label}</span>
      <span class={`font-medium tabular-nums ${colorClass()}`}>{props.value}</span>
    </div>
  );
}
