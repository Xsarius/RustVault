/**
 * Auto-Rules management page — list, create, edit, toggle, and test rules.
 */

import {
  createSignal,
  createResource,
  For,
  Show,
} from "solid-js";
import {
  Wand2,
  Plus,
  Pencil,
  Trash2,
  Play,
  GripVertical,
  ChevronDown,
  ChevronRight,
} from "lucide-solid";
import {
  Button,
  Dialog,
  TextField,
  Select,
  Switch,
  ListSkeleton,
  showToast,
} from "~/components/ui";
import {
  api,
  type AutoRule,
  type NewAutoRule,
  type UpdateAutoRule,
  type RuleCondition,
  type RuleAction,
  type TestRuleRequest,
  type TestRuleResponse,
} from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";

// ── Data fetching ────────────────────────────────────────────

async function fetchRules(): Promise<AutoRule[]> {
  const res = await api.fetchList<AutoRule>("/api/rules");
  return res.data;
}

// ── Constants ───────────────────────────────────────────────

const CONDITION_FIELDS = [
  "description_contains",
  "description_regex",
  "payee_equals",
  "payee_contains",
  "amount_range",
  "account_id",
] as const;

const ACTION_TYPES = [
  "set_category",
  "add_tags",
  "set_payee",
  "set_metadata",
] as const;

// ── Page ─────────────────────────────────────────────────────

export default function RulesPage() {
  const t = useI18n();

  const [rules, { refetch }] = createResource(fetchRules);

  // ── Form state ────────────────────────────────────────────

  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [editingRule, setEditingRule] = createSignal<AutoRule | null>(null);
  const [formName, setFormName] = createSignal("");
  const [formPriority, setFormPriority] = createSignal(0);
  const [formConditions, setFormConditions] = createSignal<RuleCondition[]>([]);
  const [formActions, setFormActions] = createSignal<RuleAction[]>([]);
  const [saving, setSaving] = createSignal(false);

  const isEditing = () => editingRule() !== null;

  const openCreateDialog = () => {
    setEditingRule(null);
    setFormName("");
    setFormPriority((rules()?.length ?? 0) + 1);
    setFormConditions([{ field: "description_contains", value: "" }]);
    setFormActions([{ type: "set_category", value: "" }]);
    setDialogOpen(true);
  };

  const openEditDialog = (rule: AutoRule) => {
    setEditingRule(rule);
    setFormName(rule.name);
    setFormPriority(rule.priority);
    // Parse conditions and actions
    const conditions = Array.isArray(rule.conditions) ? (rule.conditions as RuleCondition[]) : [];
    const actions = Array.isArray(rule.actions) ? (rule.actions as RuleAction[]) : [];
    setFormConditions(conditions.length > 0 ? conditions : [{ field: "description_contains", value: "" }]);
    setFormActions(actions.length > 0 ? actions : [{ type: "set_category", value: "" }]);
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditingRule(null);
  };

  const handleSave = async () => {
    if (!formName().trim()) return;
    setSaving(true);
    try {
      const editing = editingRule();
      if (editing) {
        const payload: UpdateAutoRule = {
          name: formName().trim(),
          priority: formPriority(),
          conditions: formConditions(),
          actions: formActions(),
        };
        await api.updateOne<AutoRule>(`/api/rules/${editing.id}`, payload);
        showToast({ title: t("rules.toast.updated") ?? "Rule updated", variant: "success" });
      } else {
        const payload: NewAutoRule = {
          name: formName().trim(),
          priority: formPriority(),
          conditions: formConditions(),
          actions: formActions(),
        };
        await api.createOne<AutoRule>("/api/rules", payload);
        showToast({ title: t("rules.toast.created") ?? "Rule created", variant: "success" });
      }
      closeDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to save rule.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setSaving(false);
    }
  };

  // ── Toggle enabled ───────────────────────────────────────

  const handleToggle = async (rule: AutoRule) => {
    try {
      await api.updateOne<AutoRule>(`/api/rules/${rule.id}`, {
        is_enabled: !rule.is_enabled,
      });
      showToast({
        title: rule.is_enabled
          ? (t("rules.toast.disabled") ?? "Rule disabled")
          : (t("rules.toast.enabled") ?? "Rule enabled"),
        variant: "success",
      });
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to toggle rule.";
      showToast({ title: msg, variant: "error" });
    }
  };

  // ── Delete ────────────────────────────────────────────────

  const handleDelete = async (rule: AutoRule) => {
    try {
      await api.del(`/api/rules/${rule.id}`);
      showToast({ title: t("rules.toast.deleted") ?? "Rule deleted", variant: "success" });
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Delete failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  // ── Test rule ─────────────────────────────────────────────

  const [testDialogOpen, setTestDialogOpen] = createSignal(false);
  const [testDesc, setTestDesc] = createSignal("");
  const [testPayee, setTestPayee] = createSignal("");
  const [testAmount, setTestAmount] = createSignal("");
  const [testAccountId] = createSignal("");
  const [testResult, setTestResult] = createSignal<TestRuleResponse | null>(null);
  const [testing, setTesting] = createSignal(false);

  const handleTest = async () => {
    if (!testDesc() || !testAmount()) return;
    setTesting(true);
    try {
      const req: TestRuleRequest = {
        conditions: formConditions(),
        description: testDesc(),
        payee: testPayee() || undefined,
        amount: testAmount(),
        account_id: testAccountId(),
      };
      const res = await api.post<{ data: TestRuleResponse }>("/api/rules/test", req);
      setTestResult(res.data);
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Test failed.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setTesting(false);
    }
  };

  // ── Condition helpers ─────────────────────────────────────

  const addCondition = () => {
    setFormConditions((c) => [...c, { field: "description_contains", value: "" }]);
  };

  const removeCondition = (index: number) => {
    setFormConditions((c) => c.filter((_, i) => i !== index));
  };

  const updateCondition = (index: number, updates: Partial<RuleCondition>) => {
    setFormConditions((c) =>
      c.map((cond, i) => (i === index ? { ...cond, ...updates } : cond)),
    );
  };

  // ── Action helpers ────────────────────────────────────────

  const addAction = () => {
    setFormActions((a) => [...a, { type: "set_category", value: "" }]);
  };

  const removeAction = (index: number) => {
    setFormActions((a) => a.filter((_, i) => i !== index));
  };

  const updateAction = (index: number, updates: Partial<RuleAction>) => {
    setFormActions((a) =>
      a.map((act, i) => (i === index ? { ...act, ...updates } : act)),
    );
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("rules.title") ?? "Auto-Rules"}
        </h1>
        <Button variant="primary" size="sm" onClick={openCreateDialog}>
          <Plus size={16} />
          {t("rules.form.createTitle") ?? "Add Rule"}
        </Button>
      </div>

      {/* Rules list */}
      <Show
        when={rules.state === "ready"}
        fallback={<ListSkeleton />}
      >
        <Show
          when={rules()!.length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center py-16 text-center">
              <Wand2 size={48} class="text-text-tertiary mb-4" />
              <h2 class="text-lg font-semibold text-text">
                {t("rules.empty.title") ?? "No rules yet"}
              </h2>
              <p class="text-sm text-text-secondary mt-1 max-w-xs">
                {t("rules.empty.description") ?? "Create rules to automatically categorize your transactions."}
              </p>
              <Button variant="primary" size="sm" class="mt-4" onClick={openCreateDialog}>
                <Plus size={16} />
                {t("rules.form.createTitle") ?? "Add Rule"}
              </Button>
            </div>
          }
        >
          <div class="space-y-2">
            <For each={rules()}>
              {(rule) => (
                <RuleCard
                  rule={rule}
                  onEdit={() => openEditDialog(rule)}
                  onToggle={() => handleToggle(rule)}
                  onDelete={() => handleDelete(rule)}
                />
              )}
            </For>
          </div>
        </Show>
      </Show>

      {/* Create/Edit Rule Dialog */}
      <Dialog
        open={dialogOpen()}
        onOpenChange={(open) => { if (!open) closeDialog(); }}
        title={isEditing()
          ? (t("rules.form.editTitle") ?? "Edit Rule")
          : (t("rules.form.createTitle") ?? "Add Rule")}
      >
        <div class="space-y-4 pt-2 min-w-[min(32rem,90vw)]">
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2">
              <TextField
                name="ruleName"
                label={t("rules.form.name") ?? "Name"}
                value={formName()}
                onInput={(e) => setFormName(e.currentTarget.value)}
                placeholder="e.g. Spotify subscription"
                required
              />
            </div>
            <TextField
              name="rulePriority"
              label={t("rules.form.priority") ?? "Priority"}
              type="number"
              value={String(formPriority())}
              onInput={(e) => setFormPriority(parseInt(e.currentTarget.value) || 0)}
            />
          </div>

          {/* Conditions */}
          <div>
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-medium text-text">
                {t("rules.conditions.title") ?? "Conditions"}
              </h3>
              <button
                class="text-xs text-primary hover:underline cursor-pointer"
                onClick={addCondition}
              >
                + {t("rules.conditions.add") ?? "Add condition"}
              </button>
            </div>
            <div class="space-y-2">
              <For each={formConditions()}>
                {(cond, i) => (
                  <div class="flex items-center gap-2">
                    <Select
                      name={`condField-${i()}`}
                      label=""
                      options={CONDITION_FIELDS.map((f) => ({
                        value: f,
                        label: (t as any)(`rules.conditions.fields.${f}`) ?? f.replace(/_/g, " "),
                      }))}
                      value={cond.field}
                      onChange={(v) => updateCondition(i(), { field: v })}
                    />
                    <TextField
                      name={`condValue-${i()}`}
                      label=""
                      value={String(cond.value ?? "")}
                      onInput={(e) => updateCondition(i(), { value: e.currentTarget.value })}
                      placeholder={t("rules.conditions.valuePlaceholder") ?? "Value"}
                    />
                    <Show when={formConditions().length > 1}>
                      <button
                        class="text-text-tertiary hover:text-danger cursor-pointer shrink-0"
                        onClick={() => removeCondition(i())}
                      >
                        <Trash2 size={14} />
                      </button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Actions */}
          <div>
            <div class="flex items-center justify-between mb-2">
              <h3 class="text-sm font-medium text-text">
                {t("rules.actions.title") ?? "Actions"}
              </h3>
              <button
                class="text-xs text-primary hover:underline cursor-pointer"
                onClick={addAction}
              >
                + {t("rules.actions.add") ?? "Add action"}
              </button>
            </div>
            <div class="space-y-2">
              <For each={formActions()}>
                {(act, i) => (
                  <div class="flex items-center gap-2">
                    <Select
                      name={`actType-${i()}`}
                      label=""
                      options={ACTION_TYPES.map((a) => ({
                        value: a,
                        label: (t as any)(`rules.actions.types.${a}`) ?? a.replace(/_/g, " "),
                      }))}
                      value={act.type}
                      onChange={(v) => updateAction(i(), { type: v })}
                    />
                    <TextField
                      name={`actValue-${i()}`}
                      label=""
                      value={String(act.value ?? "")}
                      onInput={(e) => updateAction(i(), { value: e.currentTarget.value })}
                      placeholder={t("rules.actions.valuePlaceholder") ?? "Value"}
                    />
                    <Show when={formActions().length > 1}>
                      <button
                        class="text-text-tertiary hover:text-danger cursor-pointer shrink-0"
                        onClick={() => removeAction(i())}
                      >
                        <Trash2 size={14} />
                      </button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Test button */}
          <div class="border-t border-border pt-3">
            <button
              class="text-sm text-primary hover:underline cursor-pointer flex items-center gap-1"
              onClick={() => setTestDialogOpen((v) => !v)}
            >
              <Play size={14} />
              {t("rules.test.title") ?? "Test this rule"}
            </button>

            <Show when={testDialogOpen()}>
              <div class="mt-3 space-y-2 rounded-[var(--radius-md)] border border-border p-3">
                <div class="grid grid-cols-2 gap-2">
                  <TextField
                    name="testDesc"
                    label={t("rules.test.description") ?? "Description"}
                    value={testDesc()}
                    onInput={(e) => setTestDesc(e.currentTarget.value)}
                  />
                  <TextField
                    name="testPayee"
                    label={t("rules.test.payee") ?? "Payee"}
                    value={testPayee()}
                    onInput={(e) => setTestPayee(e.currentTarget.value)}
                  />
                </div>
                <TextField
                  name="testAmount"
                  label={t("rules.test.amount") ?? "Amount"}
                  type="number"
                  value={testAmount()}
                  onInput={(e) => setTestAmount(e.currentTarget.value)}
                />
                <div class="flex items-center gap-2">
                  <Button variant="secondary" size="sm" onClick={handleTest} loading={testing()}>
                    <Play size={14} />
                    {t("rules.test.run") ?? "Run test"}
                  </Button>
                  <Show when={testResult()}>
                    {(res) => (
                      <span class={`text-sm font-medium ${
                        res().matched ? "text-income" : "text-text-secondary"
                      }`}>
                        {res().matched
                          ? (t("rules.test.matched") ?? "✓ Matched")
                          : (t("rules.test.notMatched") ?? "✗ No match")}
                      </span>
                    )}
                  </Show>
                </div>
              </div>
            </Show>
          </div>

          {/* Dialog actions */}
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={closeDialog}>
              {t("common.actions.cancel") ?? "Cancel"}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSave}
              loading={saving()}
              disabled={!formName().trim()}
            >
              {isEditing()
                ? (t("common.actions.save") ?? "Save")
                : (t("common.actions.create") ?? "Create")}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── RuleCard ────────────────────────────────────────────────

function RuleCard(props: {
  rule: AutoRule;
  onEdit: () => void;
  onToggle: () => void;
  onDelete: () => void;
}) {
  const t = useI18n();
  const [expanded, setExpanded] = createSignal(false);

  const conditions = () =>
    Array.isArray(props.rule.conditions) ? (props.rule.conditions as RuleCondition[]) : [];
  const actions = () =>
    Array.isArray(props.rule.actions) ? (props.rule.actions as RuleAction[]) : [];

  return (
    <div class={`rounded-[var(--radius-lg)] border border-border bg-surface transition-opacity ${
      props.rule.is_enabled ? "" : "opacity-60"
    }`}>
      <div class="flex items-center gap-3 px-4 py-3">
        <GripVertical size={16} class="text-text-tertiary shrink-0 cursor-grab" />

        <button
          class="text-text-tertiary hover:text-text cursor-pointer shrink-0"
          onClick={() => setExpanded((v) => !v)}
        >
          {expanded() ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </button>

        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-text truncate">{props.rule.name}</p>
          <p class="text-xs text-text-tertiary">
            {conditions().length} {t("rules.conditions.title") ?? "conditions"} · {actions().length} {t("rules.actions.title") ?? "actions"} · P{props.rule.priority}
          </p>
        </div>

        <Switch
          label=""
          checked={props.rule.is_enabled}
          onChange={props.onToggle}
        />

        <button
          class="text-text-tertiary hover:text-text cursor-pointer"
          onClick={props.onEdit}
        >
          <Pencil size={14} />
        </button>
        <button
          class="text-text-tertiary hover:text-danger cursor-pointer"
          onClick={props.onDelete}
        >
          <Trash2 size={14} />
        </button>
      </div>

      {/* Expanded detail */}
      <Show when={expanded()}>
        <div class="px-4 pb-3 pt-0 border-t border-border space-y-2">
          <Show when={conditions().length > 0}>
            <div>
              <p class="text-xs font-medium text-text-secondary mb-1">
                {t("rules.conditions.title") ?? "Conditions"}
              </p>
              <For each={conditions()}>
                {(c) => (
                  <p class="text-xs text-text-tertiary">
                    <span class="font-medium">{c.field}</span>: {String(c.value)}
                  </p>
                )}
              </For>
            </div>
          </Show>
          <Show when={actions().length > 0}>
            <div>
              <p class="text-xs font-medium text-text-secondary mb-1">
                {t("rules.actions.title") ?? "Actions"}
              </p>
              <For each={actions()}>
                {(a) => (
                  <p class="text-xs text-text-tertiary">
                    <span class="font-medium">{a.type}</span>: {String(a.value)}
                  </p>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
