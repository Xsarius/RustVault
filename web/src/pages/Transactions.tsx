/**
 * Transactions page — filterable, paginated list with bulk operations.
 *
 * Uses cursor-based pagination, debounced search, and inline quick-edit.
 */

import {
  createSignal,
  createResource,
  createEffect,
  createMemo,
  For,
  Show,
  batch,
  on,
} from "solid-js";
import { useSearchParams } from "@solidjs/router";
import {
  ArrowLeftRight,
  Plus,
  Search,
  Filter,
  Upload,
  CheckCircle2,
  Eye,
  Trash2,
  FolderTree,
  X,
} from "lucide-solid";
import {
  Button,
  TextField,
  Select,
  Checkbox,
  ListSkeleton,
  showToast,
  Dialog,
} from "~/components/ui";
import {
  api,
  type Transaction,
  type TransactionListQuery,
  type PaginatedResponse,
  type Account,
  type Bank,
  type Category,
  type Tag,
  type NewTransaction,
} from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";
import { ImportWizard } from "~/components/ImportWizard";

// ── Data fetching helpers ────────────────────────────────────

async function fetchAccounts(): Promise<Account[]> {
  const res = await api.fetchList<Account>("/api/accounts");
  return res.data;
}

async function fetchCategories(): Promise<Category[]> {
  const res = await api.fetchList<Category>("/api/categories");
  return res.data;
}

async function fetchTags(): Promise<Tag[]> {
  const res = await api.fetchList<Tag>("/api/tags");
  return res.data;
}

async function fetchBanks(): Promise<Bank[]> {
  const res = await api.fetchList<Bank>("/api/banks");
  return res.data;
}

// ── Page ─────────────────────────────────────────────────────

export default function TransactionsPage() {
  const t = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();

  // Reference data
  const [accounts] = createResource(fetchAccounts);
  const [categories] = createResource(fetchCategories);
  const [tags] = createResource(fetchTags);
  const [banks] = createResource(fetchBanks);

  // ── Filters ───────────────────────────────────────────────

  const [searchText, setSearchText] = createSignal("");
  const [filterAccount, setFilterAccount] = createSignal("");
  const [filterCategory, setFilterCategory] = createSignal("");
  const [filterType, setFilterType] = createSignal("");
  const [filterReviewed, setFilterReviewed] = createSignal("");
  const [filterDateFrom, setFilterDateFrom] = createSignal("");
  const [filterDateTo, setFilterDateTo] = createSignal("");
  const [showFilters, setShowFilters] = createSignal(false);

  // Debounced search
  const [debouncedSearch, setDebouncedSearch] = createSignal("");
  let searchTimer: ReturnType<typeof setTimeout>;
  const handleSearchInput = (value: string) => {
    setSearchText(value);
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => setDebouncedSearch(value), 300);
  };

  // Build query from filters
  const query = createMemo((): TransactionListQuery => {
    const q: TransactionListQuery = { limit: 50 };
    const s = debouncedSearch();
    if (s) q.q = s;
    const acct = filterAccount();
    if (acct) q.account_id = acct;
    const cat = filterCategory();
    if (cat) q.category_id = cat;
    const typ = filterType();
    if (typ === "income" || typ === "expense" || typ === "transfer") q.transaction_type = typ;
    const rev = filterReviewed();
    if (rev === "true") q.is_reviewed = true;
    if (rev === "false") q.is_reviewed = false;
    const df = filterDateFrom();
    if (df) q.date_from = df;
    const dt = filterDateTo();
    if (dt) q.date_to = dt;
    return q;
  });

  // ── Transactions data ─────────────────────────────────────

  const [transactions, setTransactions] = createSignal<Transaction[]>([]);
  const [cursor, setCursor] = createSignal<string | undefined>();
  const [hasMore, setHasMore] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [loadingMore, setLoadingMore] = createSignal(false);

  const loadTransactions = async (append = false) => {
    if (append) setLoadingMore(true);
    else setLoading(true);

    try {
      const q = { ...query() };
      if (append && cursor()) q.cursor = cursor();

      const params = new URLSearchParams();
      for (const [k, v] of Object.entries(q)) {
        if (v !== undefined && v !== null && v !== "") params.set(k, String(v));
      }

      const res = await api.get<PaginatedResponse<Transaction>>(
        `/api/transactions?${params.toString()}`,
      );

      if (append) {
        setTransactions((prev) => [...prev, ...res.data]);
      } else {
        setTransactions(res.data);
      }
      setCursor(res.meta.next_cursor ?? undefined);
      setHasMore(res.meta.has_more);
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to load transactions.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  };

  // Reload when filters change
  createEffect(on(query, () => {
    loadTransactions(false);
  }));

  const handleLoadMore = () => {
    if (hasMore() && !loadingMore()) loadTransactions(true);
  };

  // ── Bulk selection ────────────────────────────────────────

  const [selectedIds, setSelectedIds] = createSignal<Set<string>>(new Set());

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedIds().size === transactions().length) {
      setSelectedIds(new Set<string>());
    } else {
      setSelectedIds(new Set<string>(transactions().map((tx) => tx.id)));
    }
  };

  const clearSelection = () => setSelectedIds(new Set<string>());

  const bulkAction = async (updates: { category_id?: string; is_reviewed?: boolean; add_tag_ids?: string[] }) => {
    const ids = [...selectedIds()];
    if (ids.length === 0) return;

    try {
      await api.patch("/api/transactions/bulk", {
        transaction_ids: ids,
        ...updates,
      });
      showToast({
        title: t("transactions.toast.bulkUpdated") ?? `Updated ${ids.length} transactions`,
        variant: "success",
      });
      clearSelection();
      loadTransactions(false);
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Bulk update failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  const handleBulkReview = () => bulkAction({ is_reviewed: true });

  const handleBulkDelete = async () => {
    const ids = [...selectedIds()];
    if (ids.length === 0) return;
    try {
      await Promise.all(ids.map((id) => api.del(`/api/transactions/${id}`)));
      showToast({
        title: t("transactions.toast.bulkDeleted") ?? `Deleted ${ids.length} transactions`,
        variant: "success",
      });
      clearSelection();
      loadTransactions(false);
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Bulk delete failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  // ── Bulk categorize dialog ────────────────────────────────

  const [bulkCatDialogOpen, setBulkCatDialogOpen] = createSignal(false);
  const [bulkCategoryId, setBulkCategoryId] = createSignal("");

  const handleBulkCategorize = async () => {
    if (!bulkCategoryId()) return;
    await bulkAction({ category_id: bulkCategoryId() });
    setBulkCatDialogOpen(false);
    setBulkCategoryId("");
  };

  // ── Create transaction dialog ─────────────────────────────

  const [createDialogOpen, setCreateDialogOpen] = createSignal(false);
  const [newTx, setNewTx] = createSignal<Partial<NewTransaction>>({
    transaction_type: "expense",
  });
  const [creating, setCreating] = createSignal(false);

  const handleCreate = async () => {
    const tx = newTx();
    if (!tx.account_id || !tx.amount || !tx.date || !tx.description) return;
    setCreating(true);
    try {
      await api.createOne<Transaction>("/api/transactions", tx);
      showToast({ title: t("transactions.toast.created") ?? "Transaction created", variant: "success" });
      setCreateDialogOpen(false);
      setNewTx({ transaction_type: "expense" });
      loadTransactions(false);
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to create transaction.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setCreating(false);
    }
  };

  // ── Import wizard ─────────────────────────────────────────

  const [importWizardOpen, setImportWizardOpen] = createSignal(false);

  // ── Transaction detail ────────────────────────────────────

  const [detailTxId, setDetailTxId] = createSignal<string | null>(null);

  // ── Quick filter from query param ─────────────────────────

  createEffect(() => {
    if (searchParams.unreviewed === "true") {
      setFilterReviewed("false");
    }
  });

  // ── Helpers ───────────────────────────────────────────────

  const accountName = (accountId: string) => {
    const acct = accounts()?.find((a) => a.id === accountId);
    if (!acct) return "—";
    const bank = banks()?.find((b) => b.id === acct.bank_id);
    return bank ? `${bank.name} / ${acct.name}` : acct.name;
  };

  const categoryName = (catId: string | null) => {
    if (!catId) return "—";
    return categories()?.find((c) => c.id === catId)?.name ?? "—";
  };

  const activeFilterCount = createMemo(() => {
    let count = 0;
    if (filterAccount()) count++;
    if (filterCategory()) count++;
    if (filterType()) count++;
    if (filterReviewed()) count++;
    if (filterDateFrom() || filterDateTo()) count++;
    return count;
  });

  const categoryOptions = () =>
    [{ value: "", label: t("transactions.filters.allCategories") ?? "All" },
      ...(categories()?.map((c) => ({ value: c.id, label: c.name })) ?? [])];

  const accountOptions = () =>
    [{ value: "", label: t("transactions.filters.allAccounts") ?? "All" },
      ...(accounts()?.map((a) => ({ value: a.id, label: a.name })) ?? [])];

  const typeOptions = () => [
    { value: "", label: t("transactions.filters.allTypes") ?? "All" },
    { value: "income", label: t("transactions.types.income") ?? "Income" },
    { value: "expense", label: t("transactions.types.expense") ?? "Expense" },
    { value: "transfer", label: t("transactions.types.transfer") ?? "Transfer" },
  ];

  const reviewedOptions = () => [
    { value: "", label: t("transactions.filters.allReviewed") ?? "All" },
    { value: "true", label: t("transactions.filters.reviewed") ?? "Reviewed" },
    { value: "false", label: t("transactions.filters.unreviewed") ?? "Unreviewed" },
  ];

  const clearFilters = () => {
    batch(() => {
      setSearchText("");
      setDebouncedSearch("");
      setFilterAccount("");
      setFilterCategory("");
      setFilterType("");
      setFilterReviewed("");
      setFilterDateFrom("");
      setFilterDateTo("");
    });
    if (searchParams.unreviewed) {
      setSearchParams({ unreviewed: undefined });
    }
  };

  return (
    <div class="space-y-4">
      {/* Header */}
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("common.nav.transactions") ?? "Transactions"}
        </h1>
        <div class="flex items-center gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={(e) => {
              e.currentTarget.blur();
              setImportWizardOpen(true);
            }}
          >
            <Upload size={16} />
            {t("import.wizard.title") ?? "Import"}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={(e) => {
              e.currentTarget.blur();
              setCreateDialogOpen(true);
            }}
          >
            <Plus size={16} />
            {t("transactions.form.createTitle") ?? "Add"}
          </Button>
        </div>
      </div>

      {/* Search bar + filter toggle */}
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 text-text-tertiary" />
          <input
            type="text"
            class="w-full rounded-[var(--radius-md)] border border-border bg-surface pl-9 pr-3 py-2 text-sm text-text placeholder:text-text-tertiary focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            placeholder={t("transactions.search") ?? "Search transactions…"}
            value={searchText()}
            onInput={(e) => handleSearchInput(e.currentTarget.value)}
          />
        </div>
        <Button
          variant={showFilters() ? "primary" : "secondary"}
          size="sm"
          onClick={() => setShowFilters((v) => !v)}
        >
          <Filter size={16} />
          <Show when={activeFilterCount() > 0}>
            <span class="ml-1 text-xs bg-primary/20 text-primary rounded-full px-1.5">
              {activeFilterCount()}
            </span>
          </Show>
        </Button>
        <Button
          variant={filterReviewed() === "false" ? "primary" : "secondary"}
          size="sm"
          onClick={() => {
            setFilterReviewed((v) => (v === "false" ? "" : "false"));
          }}
        >
          <Eye size={16} />
          {t("transactions.quickFilter.unreviewed") ?? "Unreviewed"}
        </Button>
      </div>

      {/* Filters panel */}
      <Show when={showFilters()}>
        <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4 space-y-3">
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            <Select
              name="filterAccount"
              label={t("transactions.filters.account") ?? "Account"}
              options={accountOptions()}
              value={filterAccount()}
              onChange={setFilterAccount}
            />
            <Select
              name="filterCategory"
              label={t("transactions.filters.category") ?? "Category"}
              options={categoryOptions()}
              value={filterCategory()}
              onChange={setFilterCategory}
            />
            <Select
              name="filterType"
              label={t("transactions.filters.type") ?? "Type"}
              options={typeOptions()}
              value={filterType()}
              onChange={setFilterType}
            />
            <Select
              name="filterReviewed"
              label={t("transactions.filters.reviewed") ?? "Reviewed"}
              options={reviewedOptions()}
              value={filterReviewed()}
              onChange={setFilterReviewed}
            />
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <TextField
              name="dateFrom"
              label={t("transactions.filters.dateFrom") ?? "From"}
              type="date"
              value={filterDateFrom()}
              onInput={(e) => setFilterDateFrom(e.currentTarget.value)}
            />
            <TextField
              name="dateTo"
              label={t("transactions.filters.dateTo") ?? "To"}
              type="date"
              value={filterDateTo()}
              onInput={(e) => setFilterDateTo(e.currentTarget.value)}
            />
          </div>
          <Show when={activeFilterCount() > 0}>
            <div class="flex justify-end">
              <Button variant="secondary" size="sm" onClick={clearFilters}>
                <X size={14} />
                {t("transactions.filters.clear") ?? "Clear filters"}
              </Button>
            </div>
          </Show>
        </div>
      </Show>

      {/* Bulk action bar */}
      <Show when={selectedIds().size > 0}>
        <div class="flex items-center gap-3 rounded-[var(--radius-md)] border border-primary/30 bg-primary/5 px-4 py-2">
          <span class="text-sm font-medium text-text">
            {selectedIds().size} {t("transactions.bulk.selected") ?? "selected"}
          </span>
          <div class="flex-1" />
          <Button variant="secondary" size="sm" onClick={handleBulkReview}>
            <CheckCircle2 size={14} />
            {t("transactions.bulk.markReviewed") ?? "Mark Reviewed"}
          </Button>
          <Button variant="secondary" size="sm" onClick={() => setBulkCatDialogOpen(true)}>
            <FolderTree size={14} />
            {t("transactions.bulk.categorize") ?? "Categorize"}
          </Button>
          <Button variant="danger" size="sm" onClick={handleBulkDelete}>
            <Trash2 size={14} />
            {t("transactions.bulk.delete") ?? "Delete"}
          </Button>
          <button
            class="text-text-tertiary hover:text-text cursor-pointer"
            onClick={clearSelection}
          >
            <X size={16} />
          </button>
        </div>
      </Show>

      {/* Transaction list */}
      <Show
        when={!loading()}
        fallback={<ListSkeleton />}
      >
        <Show
          when={transactions().length > 0}
          fallback={
            <EmptyState
              title={t("transactions.empty.title") ?? "No transactions"}
              description={t("transactions.empty.description") ?? "Add transactions manually or import from a file."}
              onImport={() => setImportWizardOpen(true)}
              onCreate={() => setCreateDialogOpen(true)}
            />
          }
        >
          <div class="rounded-[var(--radius-lg)] border border-border bg-surface overflow-hidden">
            {/* Compact header */}
            <div class="flex items-center gap-2 px-4 py-2 border-b border-border bg-surface text-xs font-medium text-text-secondary">
              <div class="w-8">
                <Checkbox
                  label=""
                  checked={selectedIds().size === transactions().length && transactions().length > 0}
                  onChange={toggleSelectAll}
                />
              </div>
              <div class="w-24">{t("transactions.columns.date") ?? "Date"}</div>
              <div class="flex-1">{t("transactions.columns.description") ?? "Description"}</div>
              <div class="w-32 hidden sm:block">{t("transactions.columns.category") ?? "Category"}</div>
              <div class="w-36 hidden md:block">{t("transactions.columns.account") ?? "Account"}</div>
              <div class="w-24 text-right">{t("transactions.columns.amount") ?? "Amount"}</div>
              <div class="w-8" />
            </div>

            {/* Rows */}
            <For each={transactions()}>
              {(tx) => (
                <TransactionRow
                  tx={tx}
                  selected={selectedIds().has(tx.id)}
                  onToggleSelect={() => toggleSelect(tx.id)}
                  onClick={() => setDetailTxId(tx.id)}
                  categoryName={categoryName(tx.category_id)}
                  accountName={accountName(tx.account_id)}
                />
              )}
            </For>
          </div>

          {/* Load more */}
          <div class="flex justify-center py-2">
            <Show
              when={hasMore()}
              fallback={
                <p class="text-xs text-text-tertiary">
                  {t("transactions.endOfList") ?? "End of list"}
                </p>
              }
            >
              <Button
                variant="secondary"
                size="sm"
                onClick={handleLoadMore}
                loading={loadingMore()}
              >
                {t("transactions.loadMore") ?? "Load more"}
              </Button>
            </Show>
          </div>
        </Show>
      </Show>

      {/* Import wizard */}
      <ImportWizard
        open={importWizardOpen()}
        onOpenChange={setImportWizardOpen}
        onComplete={() => loadTransactions(false)}
      />

      {/* Create transaction dialog */}
      <Dialog
        open={createDialogOpen()}
        onOpenChange={(open) => { if (!open) setCreateDialogOpen(false); }}
        title={t("transactions.form.createTitle") ?? "Add Transaction"}
      >
        <div class="space-y-4 pt-2 min-w-[min(28rem,90vw)]">
          <Select
            name="txAccount"
            label={t("transactions.form.account") ?? "Account"}
            options={accounts()?.map((a) => ({ value: a.id, label: a.name })) ?? []}
            value={newTx().account_id ?? ""}
            onChange={(v) => setNewTx((tx) => ({ ...tx, account_id: v }))}
            required
          />
          <Select
            name="txType"
            label={t("transactions.form.type") ?? "Type"}
            options={[
              { value: "expense", label: t("transactions.types.expense") ?? "Expense" },
              { value: "income", label: t("transactions.types.income") ?? "Income" },
              { value: "transfer", label: t("transactions.types.transfer") ?? "Transfer" },
            ]}
            value={newTx().transaction_type ?? "expense"}
            onChange={(v) => setNewTx((tx) => ({ ...tx, transaction_type: v as "income" | "expense" | "transfer" }))}
          />
          <div class="grid grid-cols-2 gap-3">
            <TextField
              name="txAmount"
              label={t("transactions.form.amount") ?? "Amount"}
              type="number"
              value={newTx().amount ?? ""}
              onInput={(e) => setNewTx((tx) => ({ ...tx, amount: e.currentTarget.value }))}
              required
            />
            <TextField
              name="txDate"
              label={t("transactions.form.date") ?? "Date"}
              type="date"
              value={newTx().date ?? ""}
              onInput={(e) => setNewTx((tx) => ({ ...tx, date: e.currentTarget.value }))}
              required
            />
          </div>
          <TextField
            name="txDescription"
            label={t("transactions.form.description") ?? "Description"}
            value={newTx().description ?? ""}
            onInput={(e) => setNewTx((tx) => ({ ...tx, description: e.currentTarget.value }))}
            required
          />
          <TextField
            name="txPayee"
            label={t("transactions.form.payee") ?? "Payee"}
            value={newTx().payee ?? ""}
            onInput={(e) => setNewTx((tx) => ({ ...tx, payee: e.currentTarget.value }))}
          />
          <Select
            name="txCategory"
            label={t("transactions.form.category") ?? "Category"}
            options={[
              { value: "", label: "—" },
              ...(categories()?.map((c) => ({ value: c.id, label: c.name })) ?? []),
            ]}
            value={newTx().category_id ?? ""}
            onChange={(v) => setNewTx((tx) => ({ ...tx, category_id: v || undefined }))}
          />
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={() => setCreateDialogOpen(false)}>
              {t("common.actions.cancel") ?? "Cancel"}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleCreate}
              loading={creating()}
              disabled={!newTx().account_id || !newTx().amount || !newTx().date || !newTx().description}
            >
              {t("common.actions.create") ?? "Create"}
            </Button>
          </div>
        </div>
      </Dialog>

      {/* Bulk categorize dialog */}
      <Dialog
        open={bulkCatDialogOpen()}
        onOpenChange={(open) => { if (!open) setBulkCatDialogOpen(false); }}
        title={t("transactions.bulk.categorize") ?? "Set Category"}
      >
        <div class="space-y-4 pt-2">
          <p class="text-sm text-text-secondary">
            {t("transactions.bulk.categorizeDescription") ??
              `Assign a category to ${selectedIds().size} selected transactions.`}
          </p>
          <Select
            name="bulkCategory"
            label={t("transactions.form.category") ?? "Category"}
            options={categories()?.map((c) => ({ value: c.id, label: c.name })) ?? []}
            value={bulkCategoryId()}
            onChange={setBulkCategoryId}
            required
          />
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={() => setBulkCatDialogOpen(false)}>
              {t("common.actions.cancel") ?? "Cancel"}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleBulkCategorize}
              disabled={!bulkCategoryId()}
            >
              {t("common.actions.save") ?? "Save"}
            </Button>
          </div>
        </div>
      </Dialog>

      {/* Transaction detail slide-over */}
      <Show when={detailTxId()}>
        {(id) => (
          <TransactionDetail
            transactionId={id()}
            onClose={() => setDetailTxId(null)}
            onUpdated={() => loadTransactions(false)}
            categories={categories() ?? []}
            accounts={accounts() ?? []}
            tags={tags() ?? []}
          />
        )}
      </Show>
    </div>
  );
}

// ── TransactionRow ──────────────────────────────────────────

function TransactionRow(props: {
  tx: Transaction;
  selected: boolean;
  onToggleSelect: () => void;
  onClick: () => void;
  categoryName: string;
  accountName: string;
}) {
  const isIncome = () => parseFloat(props.tx.amount) >= 0 || props.tx.transaction_type === "income";

  return (
    <div
      class={`flex items-center gap-2 px-4 py-2.5 border-b border-border last:border-b-0 transition-colors cursor-pointer ${
        props.selected ? "bg-primary/5" : "hover:bg-surface-hover"
      }`}
      onClick={props.onClick}
    >
      <div class="w-8" onClick={(e) => e.stopPropagation()}>
        <Checkbox
          label=""
          checked={props.selected}
          onChange={props.onToggleSelect}
        />
      </div>
      <div class="w-24 text-sm text-text-secondary tabular-nums">
        {props.tx.date}
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm text-text truncate">{props.tx.description}</p>
        <Show when={props.tx.payee}>
          <p class="text-xs text-text-tertiary truncate">{props.tx.payee}</p>
        </Show>
      </div>
      <div class="w-32 hidden sm:block">
        <span class="text-xs text-text-secondary truncate block">{props.categoryName}</span>
      </div>
      <div class="w-36 hidden md:block">
        <span class="text-xs text-text-secondary truncate block">{props.accountName}</span>
      </div>
      <div class={`w-24 text-right text-sm font-medium tabular-nums ${
        isIncome() ? "text-income" : "text-expense"
      }`}>
        {props.tx.currency} {props.tx.amount}
      </div>
      <div class="w-8 flex items-center justify-center">
        <Show when={props.tx.is_reviewed}>
          <CheckCircle2 size={14} class="text-income" />
        </Show>
      </div>
    </div>
  );
}

// ── TransactionDetail (slide-over) ──────────────────────────

function TransactionDetail(props: {
  transactionId: string;
  onClose: () => void;
  onUpdated: () => void;
  categories: Category[];
  accounts: Account[];
  tags: Tag[];
}) {
  const t = useI18n();

  const [tx, { refetch }] = createResource(
    () => props.transactionId,
    async (id) => {
      return api.fetchOne<Transaction>(`/api/transactions/${id}`);
    },
  );

  const [saving, setSaving] = createSignal(false);
  const [editDesc, setEditDesc] = createSignal("");
  const [editPayee, setEditPayee] = createSignal("");
  const [editCategory, setEditCategory] = createSignal("");
  const [editNotes, setEditNotes] = createSignal("");

  // Populate edit fields when tx loads
  createEffect(() => {
    const t = tx();
    if (t) {
      setEditDesc(t.description);
      setEditPayee(t.payee ?? "");
      setEditCategory(t.category_id ?? "");
      setEditNotes(t.notes ?? "");
    }
  });

  const handleSave = async () => {
    const current = tx();
    if (!current) return;
    setSaving(true);
    try {
      await api.updateOne<Transaction>(`/api/transactions/${current.id}`, {
        description: editDesc(),
        payee: editPayee() || null,
        category_id: editCategory() || null,
        notes: editNotes() || null,
      });
      showToast({ title: t("transactions.toast.updated") ?? "Transaction updated", variant: "success" });
      refetch();
      props.onUpdated();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Update failed.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleToggleReview = async () => {
    const current = tx();
    if (!current) return;
    try {
      await api.updateOne<Transaction>(`/api/transactions/${current.id}`, {
        is_reviewed: !current.is_reviewed,
      });
      showToast({
        title: current.is_reviewed
          ? (t("transactions.toast.unreviewedTransaction") ?? "Marked as unreviewed")
          : (t("transactions.toast.reviewedTransaction") ?? "Marked as reviewed"),
        variant: "success",
      });
      refetch();
      props.onUpdated();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Update failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  const handleDelete = async () => {
    const current = tx();
    if (!current) return;
    try {
      await api.del(`/api/transactions/${current.id}`);
      showToast({ title: t("transactions.toast.deleted") ?? "Transaction deleted", variant: "success" });
      props.onClose();
      props.onUpdated();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Delete failed.";
      showToast({ title: msg, variant: "error" });
    }
  };

  return (
    <div class="fixed inset-y-0 right-0 z-[var(--z-modal)] w-full max-w-md bg-surface border-l border-border shadow-xl flex flex-col animate-slide-in">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border">
        <h2 class="text-lg font-semibold text-text">
          {t("transactions.detail.title") ?? "Transaction Detail"}
        </h2>
        <button class="text-text-tertiary hover:text-text cursor-pointer" onClick={props.onClose}>
          <X size={20} />
        </button>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <Show when={tx()} fallback={<ListSkeleton />}>
          {(txData) => (
            <>
              {/* Amount display */}
              <div class="text-center py-2">
                <p class={`text-3xl font-bold tabular-nums ${
                  parseFloat(txData().amount) >= 0 ? "text-income" : "text-expense"
                }`}>
                  {txData().currency} {txData().amount}
                </p>
                <p class="text-sm text-text-secondary mt-1">{txData().date}</p>
              </div>

              {/* Type badge */}
              <div class="flex items-center gap-2 justify-center">
                <span class={`px-2 py-0.5 rounded-full text-xs font-medium ${
                  txData().transaction_type === "income"
                    ? "bg-income/10 text-income"
                    : txData().transaction_type === "transfer"
                      ? "bg-primary/10 text-primary"
                      : "bg-expense/10 text-expense"
                }`}>
                  {txData().transaction_type}
                </span>
                <Show when={txData().is_reviewed}>
                  <span class="px-2 py-0.5 rounded-full text-xs font-medium bg-income/10 text-income">
                    {t("transactions.status.reviewed") ?? "Reviewed"}
                  </span>
                </Show>
              </div>

              {/* Original description diff */}
              <Show when={txData().original_desc && txData().original_desc !== txData().description}>
                <div class="rounded-[var(--radius-md)] border border-border p-3 text-xs">
                  <p class="text-text-tertiary mb-1">{t("transactions.detail.originalDesc") ?? "Original"}</p>
                  <p class="text-text-secondary">{txData().original_desc}</p>
                </div>
              </Show>

              {/* Editable fields */}
              <TextField
                name="editDesc"
                label={t("transactions.form.description") ?? "Description"}
                value={editDesc()}
                onInput={(e) => setEditDesc(e.currentTarget.value)}
              />
              <TextField
                name="editPayee"
                label={t("transactions.form.payee") ?? "Payee"}
                value={editPayee()}
                onInput={(e) => setEditPayee(e.currentTarget.value)}
              />
              <Select
                name="editCategory"
                label={t("transactions.form.category") ?? "Category"}
                options={[
                  { value: "", label: "—" },
                  ...props.categories.map((c) => ({ value: c.id, label: c.name })),
                ]}
                value={editCategory()}
                onChange={setEditCategory}
              />
              <TextField
                name="editNotes"
                label={t("transactions.form.notes") ?? "Notes"}
                value={editNotes()}
                onInput={(e) => setEditNotes(e.currentTarget.value)}
              />

              {/* Account & reference (read-only info) */}
              <div class="text-sm space-y-1 text-text-secondary">
                <p>
                  <span class="font-medium">{t("transactions.form.account") ?? "Account"}:</span>{" "}
                  {props.accounts.find((a) => a.id === txData().account_id)?.name ?? "—"}
                </p>
                <Show when={txData().reference}>
                  <p>
                    <span class="font-medium">{t("transactions.detail.reference") ?? "Reference"}:</span>{" "}
                    {txData().reference}
                  </p>
                </Show>
              </div>
            </>
          )}
        </Show>
      </div>

      {/* Footer actions */}
      <div class="flex items-center gap-2 px-4 py-3 border-t border-border">
        <Button variant="secondary" size="sm" onClick={handleToggleReview}>
          <CheckCircle2 size={14} />
          {tx()?.is_reviewed
            ? (t("transactions.detail.unreview") ?? "Unreview")
            : (t("transactions.detail.review") ?? "Mark Reviewed")}
        </Button>
        <div class="flex-1" />
        <Button variant="danger" size="sm" onClick={handleDelete}>
          <Trash2 size={14} />
        </Button>
        <Button variant="primary" size="sm" onClick={handleSave} loading={saving()}>
          {t("common.actions.save") ?? "Save"}
        </Button>
      </div>
    </div>
  );
}

// ── EmptyState ──────────────────────────────────────────────

function EmptyState(props: {
  title: string;
  description: string;
  onImport: () => void;
  onCreate: () => void;
}) {
  const t = useI18n();

  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <ArrowLeftRight size={48} class="text-text-tertiary mb-4" />
      <h2 class="text-lg font-semibold text-text">{props.title}</h2>
      <p class="text-sm text-text-secondary mt-1 max-w-xs">{props.description}</p>
      <div class="flex items-center gap-2 mt-4">
        <Button variant="secondary" size="sm" onClick={props.onImport}>
          <Upload size={16} />
          {t("import.wizard.title") ?? "Import"}
        </Button>
        <Button variant="primary" size="sm" onClick={props.onCreate}>
          <Plus size={16} />
          {t("transactions.form.createTitle") ?? "Add"}
        </Button>
      </div>
    </div>
  );
}
