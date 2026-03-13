/**
 * Budget page — list, create, edit, view summary, manage lines, compare budgets.
 */

import {
  createSignal,
  createResource,
  For,
  Show,
  createMemo,
} from "solid-js";
import {
  PiggyBank,
  Plus,
  Pencil,
  Trash2,
  Copy,
  ChevronRight,
  RefreshCw,
  BarChart2,
} from "lucide-solid";
import {
  Button,
  Dialog,
  TextField,
  Switch,
  Tabs,
  TabList,
  TabTrigger,
  TabContent,
  ListSkeleton,
  showToast,
} from "~/components/ui";
import {
  api,
  type Budget,
  type NewBudget,
  type UpdateBudget,
  type BudgetLine,
  type BudgetSummary,
  type Category,
  type CopyBudgetRequest,
} from "~/api";
import { ApiError } from "~/api/client";
import { useI18n, useLocale } from "~/i18n";
import { formatCurrency, formatAmount, formatDateRange } from "~/lib/format";

// ── Helpers ───────────────────────────────────────────────────

function progressColor(pct: number): string {
  if (pct >= 100) return "bg-red-500";
  if (pct >= 80) return "bg-amber-400";
  return "bg-emerald-500";
}

// ── Data fetching ─────────────────────────────────────────────

async function fetchBudgets(includeArchived: boolean): Promise<Budget[]> {
  return api.listBudgets(includeArchived);
}

async function fetchCategories(): Promise<Category[]> {
  const res = await api.fetchList<Category>("/api/categories");
  return res.data;
}

// ── Sub-components ────────────────────────────────────────────

function BudgetCard(props: {
  budget: Budget;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onCopy: () => void;
}) {
  const t = useI18n();
  const { locale } = useLocale();
  return (
    <div class="rounded-[var(--radius)] border border-border bg-surface p-4 flex items-center justify-between gap-4 hover:bg-surface-hover transition-colors cursor-pointer"
      onClick={props.onSelect}
    >
      <div class="flex items-center gap-3 min-w-0">
        <div class="rounded-full bg-primary/10 p-2 shrink-0">
          <PiggyBank size={20} class="text-primary" />
        </div>
        <div class="min-w-0">
          <p class="font-medium text-text truncate">
            {props.budget.name}
            {props.budget.is_archived && (
              <span class="ml-2 text-xs rounded-full bg-surface-hover border border-border px-2 py-0.5 text-text-secondary">
                {t("budget.page.archivedBadge") ?? "Archived"}
              </span>
            )}
            {props.budget.is_recurring && (
              <span class="ml-2 text-xs rounded-full bg-primary/10 text-primary px-2 py-0.5">
                {t("budget.page.recurringBadge") ?? "Recurring"}
              </span>
            )}
          </p>
          <p class="text-xs text-text-secondary mt-0.5">
            {formatDateRange(props.budget.period_start, props.budget.period_end, locale())} · {props.budget.currency}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <button
          class="p-1.5 rounded text-text-secondary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer"
          title={t("budget.copy.button") ?? "Copy"}
          onClick={(e) => { e.stopPropagation(); props.onCopy(); }}
        >
          <Copy size={15} />
        </button>
        <button
          class="p-1.5 rounded text-text-secondary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer"
          title={t("common.actions.edit") ?? "Edit"}
          onClick={(e) => { e.stopPropagation(); props.onEdit(); }}
        >
          <Pencil size={15} />
        </button>
        <button
          class="p-1.5 rounded text-text-secondary hover:text-danger hover:bg-surface-hover transition-colors cursor-pointer"
          title={t("common.actions.delete") ?? "Delete"}
          onClick={(e) => { e.stopPropagation(); props.onDelete(); }}
        >
          <Trash2 size={15} />
        </button>
        <ChevronRight size={16} class="text-text-tertiary ml-1" />
      </div>
    </div>
  );
}

// ── Summary view for a single selected budget ─────────────────

function BudgetDetail(props: { budget: Budget; categories: Category[]; onBack: () => void; onRefetch: () => void }) {
  const t = useI18n();
  const { locale } = useLocale();

  const [activeTab, setActiveTab] = createSignal("overview");
  const [summary, { refetch: refetchSummary }] = createResource(
    () => props.budget.id,
    (id) => api.getBudgetSummary(id),
  );
  const [lines, { refetch: refetchLines }] = createResource(
    () => props.budget.id,
    (id) => api.listBudgetLines(id),
  );

  // ── Add/Edit Line dialog ──────────────────────────────────

  const [lineDialogOpen, setLineDialogOpen] = createSignal(false);
  const [lineSaving, setLineSaving] = createSignal(false);
  const [lineCategoryId, setLineCategoryId] = createSignal("");
  const [linePlanned, setLinePlanned] = createSignal("");
  const [lineNotes, setLineNotes] = createSignal("");
  const [editingLine, setEditingLine] = createSignal<BudgetLine | null>(null);

  const openAddLine = () => {
    setEditingLine(null);
    setLineCategoryId("");
    setLinePlanned("");
    setLineNotes("");
    setLineDialogOpen(true);
  };

  const openEditLine = (line: BudgetLine) => {
    setEditingLine(line);
    setLineCategoryId(line.category_id ?? "");
    setLinePlanned(line.planned_amount);
    setLineNotes(line.notes ?? "");
    setLineDialogOpen(true);
  };

  const handleSaveLine = async () => {
    if (!linePlanned()) return;
    setLineSaving(true);
    try {
      const editing = editingLine();
      if (editing) {
        await api.updateBudgetLine(props.budget.id, editing.id, {
          planned_amount: linePlanned(),
          notes: lineNotes() || null,
        });
        showToast({ title: t("budget.toast.lineUpdated") ?? "Line updated", variant: "success" });
      } else {
        await api.addBudgetLine(props.budget.id, {
          category_id: lineCategoryId() || undefined,
          planned_amount: linePlanned(),
          notes: lineNotes() || undefined,
        });
        showToast({ title: t("budget.toast.lineAdded") ?? "Line added", variant: "success" });
      }
      setLineDialogOpen(false);
      refetchLines();
      refetchSummary();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : (t("budget.error.saveLine") ?? "Failed to save line.");
      showToast({ title: msg, variant: "error" });
    } finally {
      setLineSaving(false);
    }
  };

  const handleDeleteLine = async (line: BudgetLine) => {
    try {
      await api.deleteBudgetLine(props.budget.id, line.id);
      showToast({ title: t("budget.toast.lineDeleted") ?? "Line removed", variant: "success" });
      refetchLines();
      refetchSummary();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : (t("budget.error.deleteLine") ?? "Failed to remove.");
      showToast({ title: msg, variant: "error" });
    }
  };

  const handleRefreshActuals = async () => {
    try {
      await api.getBudgetSummary(props.budget.id);
      refetchSummary();
      refetchLines();
      showToast({ title: t("budget.toast.actualsRefreshed") ?? "Actuals refreshed", variant: "success" });
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Refresh failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  const categoryName = (id: string | null) => {
    if (!id) return "—";
    return props.categories.find((c) => c.id === id)?.name ?? id;
  };

  // ── Comparison ────────────────────────────────────────────

  const [otherBudgets] = createResource(
    () => props.budget.id,
    async () => {
      const all = await api.listBudgets(false);
      return all.filter((b) => b.id !== props.budget.id);
    },
  );
  const [compareId, setCompareId] = createSignal<string | null>(null);
  const [otherSummary] = createResource(compareId, (id) =>
    id ? api.getBudgetSummary(id) : Promise.resolve(null),
  );

  const comparedSummaryLines = createMemo(() => {
    const mine = summary();
    const other: BudgetSummary | null = otherSummary() ?? null;
    if (!mine) return [];
    return mine.lines.map((l) => {
      const match = other?.lines.find((ol) => ol.category_id === l.category_id);
      return { line: l, other: match ?? null };
    });
  });

  return (
    <div class="space-y-4">
      {/* Back + header */}
      <div class="flex items-center gap-3">
        <button
          class="text-sm text-text-secondary hover:text-text cursor-pointer"
          onClick={props.onBack}
        >
          ← {t("common.actions.back") ?? "Back"}
        </button>
        <span class="text-text-tertiary">/</span>
        <h2 class="text-xl font-semibold text-text">{props.budget.name}</h2>
        <span class="text-sm text-text-secondary">
          {formatDateRange(props.budget.period_start, props.budget.period_end, locale())} · {props.budget.currency}
        </span>
      </div>

      <Tabs value={activeTab()} onChange={setActiveTab}>
        <TabList>
          <TabTrigger value="overview">{t("budget.tabs.overview") ?? "Overview"}</TabTrigger>
          <TabTrigger value="lines">{t("budget.tabs.lines") ?? "Budget Lines"}</TabTrigger>
          <TabTrigger value="comparison">{t("budget.tabs.comparison") ?? "Comparison"}</TabTrigger>
        </TabList>

        {/* ── Overview tab ────────────────────────────────────── */}
        <TabContent value="overview">
          <Show when={summary.state === "ready"} fallback={<ListSkeleton />}>
            <Show when={summary()} keyed>
              {(s) => {
                const totalPlanned = parseFloat(s.total_planned_expenses || "0") + parseFloat(s.total_planned_income || "0");
                const totalActual = parseFloat(s.total_actual_expenses || "0") + parseFloat(s.total_actual_income || "0");
                const pct = totalPlanned > 0 ? Math.min(100, (totalActual / totalPlanned) * 100) : 0;
                return (
                  <div class="space-y-6">
                    {/* KPI row */}
                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                      <div class="rounded-[var(--radius)] border border-border bg-surface p-4">
                        <p class="text-xs text-text-secondary uppercase tracking-wide">{t("budget.summary.totalPlanned") ?? "Total Planned"}</p>
                        <p class="text-2xl font-bold text-text mt-1">
                          {formatCurrency(totalPlanned, props.budget.currency, locale())}
                        </p>
                      </div>
                      <div class="rounded-[var(--radius)] border border-border bg-surface p-4">
                        <p class="text-xs text-text-secondary uppercase tracking-wide">{t("budget.summary.totalActual") ?? "Total Actual"}</p>
                        <p class="text-2xl font-bold text-text mt-1">
                          {formatCurrency(totalActual, props.budget.currency, locale())}
                        </p>
                      </div>
                      <div class="rounded-[var(--radius)] border border-border bg-surface p-4">
                        <p class="text-xs text-text-secondary uppercase tracking-wide">{t("budget.summary.totalRemaining") ?? "Remaining"}</p>
                        <p class={`text-2xl font-bold mt-1 ${parseFloat(s.net_actual) < 0 ? "text-red-500" : "text-emerald-500"}`}>
                          {formatCurrency(s.net_actual, props.budget.currency, locale())}
                        </p>
                      </div>
                    </div>

                    {/* Overall progress bar */}
                    <div>
                      <div class="flex justify-between text-sm mb-1">
                        <span class="text-text-secondary">{t("budget.summary.percentUsed")?.replace("{{pct}}", pct.toFixed(0)) ?? `${pct.toFixed(0)}% used`}</span>
                        <span class={`text-xs font-medium ${pct >= 100 ? "text-red-500" : "text-emerald-500"}`}>
                          {pct >= 100 ? (t("budget.summary.overBudget") ?? "Over budget") : (t("budget.summary.onTrack") ?? "On track")}
                        </span>
                      </div>
                      <div class="h-2 rounded-full bg-surface-hover overflow-hidden">
                        <div
                          class={`h-full rounded-full ${progressColor(pct)} transition-all`}
                          style={{ width: `${Math.min(pct, 100)}%` }}
                        />
                      </div>
                    </div>

                    {/* Per-category breakdown */}
                    <div class="space-y-2">
                      <div class="flex items-center justify-between">
                        <h3 class="text-sm font-medium text-text">{t("budget.tabs.lines") ?? "Budget Lines"}</h3>
                        <button
                          class="text-xs text-text-secondary hover:text-text flex items-center gap-1 cursor-pointer"
                          onClick={handleRefreshActuals}
                        >
                          <RefreshCw size={12} />
                          {t("budget.lines.refresh") ?? "Refresh Actuals"}
                        </button>
                      </div>
                      <For each={s.lines}>
                        {(line) => {
                          const linePct = parseFloat(line.percent_used) || 0;
                          return (
                            <div class="rounded-[var(--radius)] border border-border bg-surface px-4 py-3">
                              <div class="flex justify-between text-sm mb-1">
                                <span class="font-medium text-text">{categoryName(line.category_id)}</span>
                                <span class="text-text-secondary">
                                  {formatCurrency(line.actual_amount, props.budget.currency, locale())} / {formatCurrency(line.planned_amount, props.budget.currency, locale())}
                                </span>
                              </div>
                              <div class="h-1.5 rounded-full bg-surface-hover overflow-hidden">
                                <div
                                  class={`h-full rounded-full ${progressColor(linePct)}`}
                                  style={{ width: `${Math.min(linePct, 100)}%` }}
                                />
                              </div>
                              <div class="flex justify-between text-xs text-text-tertiary mt-1">
                                <span>{linePct.toFixed(0)}% used</span>
                                <span>{formatCurrency(line.remaining, props.budget.currency, locale())} remaining</span>
                              </div>
                            </div>
                          );
                        }}
                      </For>
                    </div>
                  </div>
                );
              }}
            </Show>
          </Show>
        </TabContent>

        {/* ── Lines tab ────────────────────────────────────────── */}
        <TabContent value="lines">
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-semibold text-text">{t("budget.lines.title") ?? "Budget Lines"}</h3>
              <Button variant="secondary" size="sm" onClick={openAddLine}>
                <Plus size={14} />
                {t("budget.lines.addLine") ?? "Add Category"}
              </Button>
            </div>
            <Show when={lines.state === "ready"} fallback={<ListSkeleton />}>
              <Show
                when={(lines()?.length ?? 0) > 0}
                fallback={
                  <p class="text-sm text-text-secondary py-8 text-center">
                    {t("budget.lines.empty") ?? "No budget lines. Add categories to track spending."}
                  </p>
                }
              >
                <div class="space-y-2">
                  <For each={lines()}>
                    {(line) => (
                      <div class="rounded-[var(--radius)] border border-border bg-surface px-4 py-3 flex items-center justify-between gap-4">
                        <div class="min-w-0">
                          <p class="font-medium text-text text-sm">{categoryName(line.category_id)}</p>
                          <p class="text-xs text-text-secondary mt-0.5">
                            {t("budget.lines.planned") ?? "Planned"}: {formatCurrency(line.planned_amount, props.budget.currency, locale())} · {t("budget.lines.actual") ?? "Actual"}: {formatCurrency(line.actual_amount_cache, props.budget.currency, locale())}
                          </p>
                          <Show when={line.notes}>
                            <p class="text-xs text-text-tertiary">{line.notes}</p>
                          </Show>
                        </div>
                        <div class="flex items-center gap-1 shrink-0">
                          <button
                            class="p-1.5 rounded text-text-secondary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer"
                            onClick={() => openEditLine(line)}
                          >
                            <Pencil size={14} />
                          </button>
                          <button
                            class="p-1.5 rounded text-text-secondary hover:text-danger hover:bg-surface-hover transition-colors cursor-pointer"
                            onClick={() => handleDeleteLine(line)}
                          >
                            <Trash2 size={14} />
                          </button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </div>
        </TabContent>

        {/* ── Comparison tab ───────────────────────────────────── */}
        <TabContent value="comparison">
          <div class="space-y-4">
            <div>
              <p class="text-sm text-text-secondary mb-2">
                {t("budget.comparison.description") ?? "Compare this budget side-by-side with another."}
              </p>
              <Show
                when={(otherBudgets()?.length ?? 0) > 0}
                fallback={<p class="text-sm text-text-tertiary">{t("budget.comparison.noBudgets") ?? "No other budgets to compare."}</p>}
              >
                <div class="flex flex-wrap gap-2">
                  <For each={otherBudgets()}>
                    {(b) => (
                      <button
                        class={`px-3 py-1.5 rounded-full border text-sm transition-colors cursor-pointer ${compareId() === b.id ? "bg-primary text-white border-primary" : "border-border text-text-secondary hover:border-primary hover:text-text"}`}
                        onClick={() => setCompareId(compareId() === b.id ? null : b.id)}
                      >
                        {b.name}
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </div>

            <Show when={compareId()}>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b border-border text-text-secondary">
                      <th class="text-left py-2 pr-4">{t("budget.comparison.category") ?? "Category"}</th>
                      <th class="text-right py-2 px-2">{t("budget.comparison.thisPlanned") ?? "Planned"}</th>
                      <th class="text-right py-2 px-2">{t("budget.comparison.thisActual") ?? "Actual"}</th>
                      <th class="text-right py-2 px-2">{t("budget.comparison.otherPlanned") ?? "Other Planned"}</th>
                      <th class="text-right py-2 px-2">{t("budget.comparison.otherActual") ?? "Other Actual"}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={comparedSummaryLines()}>
                      {({ line, other }) => (
                        <tr class="border-b border-border last:border-0">
                          <td class="py-2 pr-4 font-medium text-text">{categoryName(line.category_id)}</td>
                          <td class="text-right py-2 px-2 text-text">{formatAmount(line.planned_amount, 2, locale())}</td>
                          <td class="text-right py-2 px-2 text-text">{formatAmount(line.actual_amount, 2, locale())}</td>
                          <td class="text-right py-2 px-2 text-text-secondary">{other ? formatAmount(other.planned_amount, 2, locale()) : "—"}</td>
                          <td class="text-right py-2 px-2 text-text-secondary">{other ? formatAmount(other.actual_amount, 2, locale()) : "—"}</td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </Show>
          </div>
        </TabContent>
      </Tabs>

      {/* Add/Edit Line dialog */}
      <Dialog
        open={lineDialogOpen()}
        onOpenChange={(open) => { if (!open) setLineDialogOpen(false); }}
        title={editingLine() ? (t("budget.lines.editTitle") ?? "Edit Budget Line") : (t("budget.lines.addTitle") ?? "Add Budget Line")}
      >
        <div class="space-y-4 pt-2 min-w-[min(28rem,90vw)]">
          <Show when={!editingLine()}>
            <div>
              <label class="block text-sm font-medium text-text mb-1">
                {t("budget.lines.category") ?? "Category"}
              </label>
              <select
                class="w-full rounded-[var(--radius-sm)] border border-border bg-surface text-text px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/30"
                value={lineCategoryId()}
                onChange={(e) => setLineCategoryId(e.currentTarget.value)}
              >
                <option value="">{t("budget.lines.selectCategory") ?? "Select a category"}</option>
                <For each={props.categories}>
                  {(c) => <option value={c.id}>{c.name}</option>}
                </For>
              </select>
            </div>
          </Show>
          <TextField
            name="linePlanned"
            label={t("budget.lines.planned") ?? "Planned Amount"}
            type="number"
            value={linePlanned()}
            onInput={(e) => setLinePlanned(e.currentTarget.value)}
            placeholder="0.00"
            required
          />
          <TextField
            name="lineNotes"
            label={t("budget.lines.notes") ?? "Notes"}
            value={lineNotes()}
            onInput={(e) => setLineNotes(e.currentTarget.value)}
            placeholder={t("budget.lines.notesPlaceholder") ?? "Optional notes…"}
          />
          <div class="flex justify-end gap-2 pt-2">
            <Button variant="secondary" size="sm" onClick={() => setLineDialogOpen(false)}>
              {t("budget.form.cancel") ?? "Cancel"}
            </Button>
            <Button variant="primary" size="sm" onClick={handleSaveLine} disabled={lineSaving()}>
              {lineSaving() ? "…" : (editingLine() ? (t("budget.form.update") ?? "Update") : (t("budget.form.create") ?? "Add"))}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────

export default function BudgetPage() {
  const t = useI18n();

  const [includeArchived, setIncludeArchived] = createSignal(false);
  const [budgets, { refetch }] = createResource(includeArchived, fetchBudgets);
  const [categories] = createResource(fetchCategories);

  // ── Selected budget (detail view) ────────────────────────

  const [selectedBudget, setSelectedBudget] = createSignal<Budget | null>(null);

  // ── Create / Edit dialog ─────────────────────────────────

  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [editingBudget, setEditingBudget] = createSignal<Budget | null>(null);
  const [saving, setSaving] = createSignal(false);

  const [formName, setFormName] = createSignal("");
  const [formCurrency, setFormCurrency] = createSignal("EUR");
  const [formStart, setFormStart] = createSignal("");
  const [formEnd, setFormEnd] = createSignal("");
  const [formRecurring, setFormRecurring] = createSignal(false);
  const [formRrule, setFormRrule] = createSignal("");
  const [formNotes, setFormNotes] = createSignal("");

  const isEditing = () => editingBudget() !== null;

  const openCreateDialog = () => {
    setEditingBudget(null);
    setFormName("");
    setFormCurrency("EUR");
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 1).padStart(2, "0");
    setFormStart(`${y}-${m}-01`);
    const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
    setFormEnd(`${y}-${m}-${lastDay}`);
    setFormRecurring(false);
    setFormRrule("");
    setFormNotes("");
    setDialogOpen(true);
  };

  const openEditDialog = (b: Budget) => {
    setEditingBudget(b);
    setFormName(b.name);
    setFormCurrency(b.currency);
    setFormStart(b.period_start);
    setFormEnd(b.period_end);
    setFormRecurring(b.is_recurring);
    setFormRrule(b.recurrence_rule ?? "");
    setFormNotes(b.notes ?? "");
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditingBudget(null);
  };

  const handleSave = async () => {
    if (!formName().trim() || !formStart() || !formEnd()) return;
    setSaving(true);
    try {
      const editing = editingBudget();
      if (editing) {
        const payload: UpdateBudget = {
          name: formName().trim(),
          currency: formCurrency(),
          period_start: formStart(),
          period_end: formEnd(),
          is_recurring: formRecurring(),
          recurrence_rule: formRecurring() ? (formRrule() || null) : null,
          notes: formNotes() || null,
        };
        await api.updateBudget(editing.id, payload);
        showToast({ title: t("budget.toast.updated") ?? "Budget updated", variant: "success" });
      } else {
        const payload: NewBudget = {
          name: formName().trim(),
          currency: formCurrency(),
          period_start: formStart(),
          period_end: formEnd(),
          is_recurring: formRecurring(),
          recurrence_rule: formRecurring() ? formRrule() : undefined,
          notes: formNotes() || undefined,
        };
        await api.createBudget(payload);
        showToast({ title: t("budget.toast.created") ?? "Budget created", variant: "success" });
      }
      closeDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : (t("budget.error.save") ?? "Failed to save budget.");
      showToast({ title: msg, variant: "error" });
    } finally {
      setSaving(false);
    }
  };

  // ── Delete ────────────────────────────────────────────────

  const handleDelete = async (b: Budget) => {
    try {
      await api.deleteBudget(b.id);
      showToast({ title: t("budget.toast.deleted") ?? "Budget deleted", variant: "success" });
      if (selectedBudget()?.id === b.id) setSelectedBudget(null);
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : (t("budget.error.delete") ?? "Delete failed.");
      showToast({ title: msg, variant: "error" });
    }
  };

  // ── Copy dialog ───────────────────────────────────────────

  const [copyTarget, setCopyTarget] = createSignal<Budget | null>(null);
  const [copyName, setCopyName] = createSignal("");
  const [copyStart, setCopyStart] = createSignal("");
  const [copyEnd, setCopyEnd] = createSignal("");
  const [copying, setCopying] = createSignal(false);

  const openCopyDialog = (b: Budget) => {
    setCopyTarget(b);
    setCopyName(`${b.name} (copy)`);
    const now = new Date();
    const y = now.getFullYear();
    const m = String(now.getMonth() + 2 > 12 ? 1 : now.getMonth() + 2).padStart(2, "0");
    const yr = now.getMonth() + 2 > 12 ? y + 1 : y;
    setCopyStart(`${yr}-${m}-01`);
    const lastDay = new Date(yr, parseInt(m), 0).getDate();
    setCopyEnd(`${yr}-${m}-${String(lastDay).padStart(2, "0")}`);
  };

  const handleCopy = async () => {
    const target = copyTarget();
    if (!target) return;
    setCopying(true);
    try {
      const payload: CopyBudgetRequest = {
        name: copyName(),
        period_start: copyStart(),
        period_end: copyEnd(),
      };
      await api.copyBudget(target.id, payload);
      showToast({ title: t("budget.toast.copied") ?? "Budget copied", variant: "success" });
      setCopyTarget(null);
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : (t("budget.error.copy") ?? "Copy failed.");
      showToast({ title: msg, variant: "error" });
    } finally {
      setCopying(false);
    }
  };

  // ── Render ────────────────────────────────────────────────

  return (
    <div class="space-y-6">
      {/* Show either list or detail */}
      <Show
        when={selectedBudget() === null}
        fallback={
          <BudgetDetail
            budget={selectedBudget()!}
            categories={categories() ?? []}
            onBack={() => setSelectedBudget(null)}
            onRefetch={refetch}
          />
        }
      >
        {/* Header */}
        <div class="flex items-center justify-between gap-4 flex-wrap">
          <h1 class="text-2xl font-bold text-text">
            {t("budget.page.title") ?? "Budgets"}
          </h1>
          <div class="flex items-center gap-3">
            <Switch
              label={t("budget.archive.button") ?? "Show Archived"}
              checked={includeArchived()}
              onChange={setIncludeArchived}
            />
            <Button variant="primary" size="sm" onClick={openCreateDialog}>
              <Plus size={16} />
              {t("budget.create.button") ?? "New Budget"}
            </Button>
          </div>
        </div>

        {/* Budget list */}
        <Show when={budgets.state === "ready"} fallback={<ListSkeleton />}>
          <Show
            when={(budgets()?.length ?? 0) > 0}
            fallback={
              <div class="flex flex-col items-center justify-center py-20 text-center">
                <BarChart2 size={48} class="text-text-tertiary mb-4" />
                <h2 class="text-lg font-semibold text-text">
                  {t("budget.page.empty") ?? "No budgets yet. Create your first budget to start planning."}
                </h2>
                <Button variant="primary" size="sm" class="mt-4" onClick={openCreateDialog}>
                  <Plus size={16} />
                  {t("budget.create.button") ?? "New Budget"}
                </Button>
              </div>
            }
          >
            <div class="space-y-2">
              <For each={budgets()}>
                {(b) => (
                  <BudgetCard
                    budget={b}
                    onSelect={() => setSelectedBudget(b)}
                    onEdit={() => openEditDialog(b)}
                    onDelete={() => handleDelete(b)}
                    onCopy={() => openCopyDialog(b)}
                  />
                )}
              </For>
            </div>
          </Show>
        </Show>
      </Show>

      {/* Create / Edit dialog */}
      <Dialog
        open={dialogOpen()}
        onOpenChange={(open) => { if (!open) closeDialog(); }}
        title={isEditing() ? (t("budget.create.editTitle") ?? "Edit Budget") : (t("budget.create.title") ?? "Create Budget")}
      >
        <div class="space-y-4 pt-2 min-w-[min(30rem,90vw)]">
          <TextField
            name="budgetName"
            label={t("budget.form.name") ?? "Name"}
            value={formName()}
            onInput={(e) => setFormName(e.currentTarget.value)}
            placeholder={t("budget.form.namePlaceholder") ?? "e.g. May 2025"}
            required
          />
          <div class="grid grid-cols-2 gap-3">
            <TextField
              name="budgetStart"
              label={t("budget.form.periodStart") ?? "Period Start"}
              type="date"
              value={formStart()}
              onInput={(e) => setFormStart(e.currentTarget.value)}
              required
            />
            <TextField
              name="budgetEnd"
              label={t("budget.form.periodEnd") ?? "Period End"}
              type="date"
              value={formEnd()}
              onInput={(e) => setFormEnd(e.currentTarget.value)}
              required
            />
          </div>
          <TextField
            name="budgetCurrency"
            label={t("budget.form.currency") ?? "Currency"}
            value={formCurrency()}
            onInput={(e) => setFormCurrency(e.currentTarget.value.toUpperCase())}
            placeholder="EUR"
          />
          <Switch
            label={t("budget.form.isRecurring") ?? "Recurring budget"}
            checked={formRecurring()}
            onChange={setFormRecurring}
          />
          <Show when={formRecurring()}>
            <TextField
              name="budgetRrule"
              label={t("budget.form.recurrenceRule") ?? "Recurrence rule (RRULE)"}
              value={formRrule()}
              onInput={(e) => setFormRrule(e.currentTarget.value)}
              placeholder={t("budget.form.recurrenceRulePlaceholder") ?? "e.g. FREQ=MONTHLY;INTERVAL=1"}
            />
          </Show>
          <TextField
            name="budgetNotes"
            label={t("budget.form.notes") ?? "Notes"}
            value={formNotes()}
            onInput={(e) => setFormNotes(e.currentTarget.value)}
            placeholder={t("budget.form.notesPlaceholder") ?? "Optional notes…"}
          />
          <div class="flex justify-end gap-2 pt-2">
            <Button variant="secondary" size="sm" onClick={closeDialog}>
              {t("budget.form.cancel") ?? "Cancel"}
            </Button>
            <Button variant="primary" size="sm" onClick={handleSave} disabled={saving()}>
              {saving() ? "…" : (isEditing() ? (t("budget.form.update") ?? "Update") : (t("budget.form.create") ?? "Create"))}
            </Button>
          </div>
        </div>
      </Dialog>

      {/* Copy dialog */}
      <Show when={copyTarget() !== null}>
        <Dialog
          open={copyTarget() !== null}
          onOpenChange={(open) => { if (!open) setCopyTarget(null); }}
          title={t("budget.copy.title") ?? "Copy Budget"}
        >
          <div class="space-y-4 pt-2 min-w-[min(28rem,90vw)]">
            <TextField
              name="copyName"
              label={t("budget.copy.newName") ?? "New budget name"}
              value={copyName()}
              onInput={(e) => setCopyName(e.currentTarget.value)}
              placeholder={t("budget.copy.newNamePlaceholder") ?? "e.g. June 2025"}
            />
            <div class="grid grid-cols-2 gap-3">
              <TextField
                name="copyStart"
                label={t("budget.form.periodStart") ?? "Period Start"}
                type="date"
                value={copyStart()}
                onInput={(e) => setCopyStart(e.currentTarget.value)}
              />
              <TextField
                name="copyEnd"
                label={t("budget.form.periodEnd") ?? "Period End"}
                type="date"
                value={copyEnd()}
                onInput={(e) => setCopyEnd(e.currentTarget.value)}
              />
            </div>
            <div class="flex justify-end gap-2 pt-2">
              <Button variant="secondary" size="sm" onClick={() => setCopyTarget(null)}>
                {t("budget.form.cancel") ?? "Cancel"}
              </Button>
              <Button variant="primary" size="sm" onClick={handleCopy} disabled={copying()}>
                {copying() ? "…" : (t("budget.copy.button") ?? "Copy")}
              </Button>
            </div>
          </div>
        </Dialog>
      </Show>
    </div>
  );
}
