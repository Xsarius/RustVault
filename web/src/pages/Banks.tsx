/**
 * Banks & Accounts page — list banks with nested accounts.
 *
 * Supports creating/editing banks and accounts via dialogs triggered
 * by URL query params (?create=true, ?create-account=true).
 */

import {
  createSignal,
  createResource,
  For,
  Show,
  Suspense,
} from "solid-js";
import { useSearchParams } from "@solidjs/router";
import {
  Building2,
  Plus,
  MoreVertical,
  ChevronDown,
  ChevronRight,
  Archive,
  Pencil,
  Trash2,
  CreditCard,
} from "lucide-solid";
import {
  Button,
  Dialog,
  TextField,
  ListSkeleton,
  DropdownMenu,
  DropdownItem,
  DropdownSeparator,
  showToast,
} from "~/components/ui";
import { Select } from "~/components/ui";
import { api, type Bank, type Account, type NewAccount, type AccountType } from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";

// ── Data fetching ────────────────────────────────────────────

async function fetchBanks(): Promise<Bank[]> {
  const res = await api.fetchList<Bank>("/api/banks");
  return res.data;
}

async function fetchAccounts(bankId: string): Promise<Account[]> {
  const res = await api.fetchList<Account>("/api/accounts");
  return res.data.filter((account) => account.bank_id === bankId);
}

// ── Page ─────────────────────────────────────────────────────

export default function BanksPage() {
  const t = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();

  const [banks, { refetch }] = createResource(fetchBanks);

  // ── Create bank dialog ─────────────────────────────────────

  const [bankDialogOpen, setBankDialogOpen] = createSignal(false);
  const [bankName, setBankName] = createSignal("");
  const [bankSaving, setBankSaving] = createSignal(false);

  // Open dialog from query param
  const showCreateBank = () => searchParams.create === "true";

  const openBankDialog = () => {
    setBankName("");
    setBankDialogOpen(true);
  };

  const closeBankDialog = () => {
    setBankDialogOpen(false);
    // Clear query param if set
    if (searchParams.create) {
      setSearchParams({ create: undefined });
    }
  };

  const handleCreateBank = async () => {
    if (!bankName().trim()) return;
    setBankSaving(true);
    try {
      await api.createOne<Bank>("/api/banks", { name: bankName().trim() });
      showToast({ title: "Bank created", variant: "success" });
      closeBankDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to create bank.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setBankSaving(false);
    }
  };

  // Auto-open dialog from query param
  if (showCreateBank()) {
    openBankDialog();
  }

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("common.nav.banks") ?? "Banks & Accounts"}
        </h1>
        <Button variant="primary" size="sm" onClick={openBankDialog}>
          <Plus size={16} />
          Add Bank
        </Button>
      </div>

      {/* Banks list */}
      <Suspense fallback={<ListSkeleton />}>
        <Show
          when={banks() && banks()!.length > 0}
          fallback={
            <EmptyState
              title="No banks yet"
              description="Add your first bank to start tracking accounts."
              onAction={openBankDialog}
            />
          }
        >
          <div class="space-y-3">
            <For each={banks()}>
              {(bank) => <BankCard bank={bank} onRefetch={refetch} />}
            </For>
          </div>
        </Show>
      </Suspense>

      {/* Create bank dialog */}
      <Dialog
        open={bankDialogOpen()}
        onOpenChange={(open) => {
          if (!open) closeBankDialog();
        }}
        title="Add Bank"
      >
        <div class="space-y-4 pt-2">
          <TextField
            name="bankName"
            label="Bank name"
            value={bankName()}
            onInput={(e) => setBankName(e.currentTarget.value)}
            placeholder="e.g. Chase, Revolut"
            required
          />
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={closeBankDialog}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleCreateBank}
              loading={bankSaving()}
              disabled={!bankName().trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── BankCard ─────────────────────────────────────────────────

const ACCOUNT_TYPE_OPTIONS: { value: AccountType; label: string }[] = [
  { value: "checking", label: "Checking" },
  { value: "savings", label: "Savings" },
  { value: "credit", label: "Credit Card" },
  { value: "investment", label: "Investment" },
  { value: "loan", label: "Loan" },
];

function BankCard(props: { bank: Bank; onRefetch: () => void }) {
  const [expanded, setExpanded] = createSignal(true);
  const [accounts, { refetch: refetchAccounts }] = createResource(
    () => props.bank.id,
    fetchAccounts,
  );

  // ── Add Account dialog ─────────────────────────────────────
  const [acctDialogOpen, setAcctDialogOpen] = createSignal(false);
  const [acctName, setAcctName] = createSignal("");
  const [acctCurrency, setAcctCurrency] = createSignal("USD");
  const [acctType, setAcctType] = createSignal<AccountType>("checking");
  const [acctSaving, setAcctSaving] = createSignal(false);

  const openAcctDialog = () => {
    setAcctName("");
    setAcctCurrency("USD");
    setAcctType("checking");
    setAcctDialogOpen(true);
  };

  const handleCreateAccount = async () => {
    if (!acctName().trim()) return;
    setAcctSaving(true);
    try {
      const body: NewAccount = {
        bank_id: props.bank.id,
        name: acctName().trim(),
        currency: acctCurrency().trim().toUpperCase(),
        type: acctType(),
      };
      await api.createOne<Account>("/api/accounts", body);
      showToast({ title: "Account created", variant: "success" });
      setAcctDialogOpen(false);
      refetchAccounts();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to create account.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setAcctSaving(false);
    }
  };

  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface">
      {/* Bank header */}
      <div class="flex items-center gap-3 px-4 py-3">
        <button
          class="text-text-tertiary hover:text-text transition-colors cursor-pointer"
          onClick={() => setExpanded((e) => !e)}
        >
          {expanded() ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </button>

        <Building2 size={18} class="text-text-secondary shrink-0" />
        <span class="font-medium text-text flex-1 truncate">
          {props.bank.name}
        </span>

        <Button variant="ghost" size="sm" onClick={openAcctDialog}>
          <Plus size={14} />
          Add Account
        </Button>

        <DropdownMenu
          trigger={
            <button class="h-7 w-7 flex items-center justify-center rounded-[var(--radius-md)] text-text-tertiary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer">
              <MoreVertical size={16} />
            </button>
          }
        >
          <DropdownItem onSelect={() => {}}>
            <Pencil size={14} />
            Edit
          </DropdownItem>
          <DropdownItem onSelect={() => {}}>
            <Archive size={14} />
            Archive
          </DropdownItem>
          <DropdownSeparator />
          <DropdownItem onSelect={() => {}} danger>
            <Trash2 size={14} />
            Delete
          </DropdownItem>
        </DropdownMenu>
      </div>

      {/* Accounts list */}
      <Show when={expanded()}>
        <div class="border-t border-border">
          <Suspense
            fallback={
              <div class="px-4 py-3 text-sm text-text-tertiary">
                Loading accounts…
              </div>
            }
          >
            <Show
              when={accounts() && accounts()!.length > 0}
              fallback={
                <div class="px-4 py-3 flex items-center justify-between">
                  <span class="text-sm text-text-tertiary">No accounts yet</span>
                  <Button variant="ghost" size="sm" onClick={openAcctDialog}>
                    <Plus size={14} />
                    Add Account
                  </Button>
                </div>
              }
            >
              <For each={accounts()}>
                {(account) => <AccountRow account={account} />}
              </For>
            </Show>
          </Suspense>
        </div>
      </Show>

      {/* Add Account dialog */}
      <Dialog
        open={acctDialogOpen()}
        onOpenChange={(open) => { if (!open) setAcctDialogOpen(false); }}
        title={`Add Account — ${props.bank.name}`}
      >
        <div class="space-y-4 pt-2">
          <TextField
            name="acctName"
            label="Account name"
            value={acctName()}
            onInput={(e) => setAcctName(e.currentTarget.value)}
            placeholder="e.g. Main Checking"
            required
          />
          <TextField
            name="acctCurrency"
            label="Currency"
            value={acctCurrency()}
            onInput={(e) => setAcctCurrency(e.currentTarget.value)}
            placeholder="USD"
            required
          />
          <Select
            name="acctType"
            label="Account type"
            options={ACCOUNT_TYPE_OPTIONS}
            value={acctType()}
            onChange={(v) => setAcctType(v as AccountType)}
            required
          />
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={() => setAcctDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleCreateAccount}
              loading={acctSaving()}
              disabled={!acctName().trim() || !acctCurrency().trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── AccountRow ───────────────────────────────────────────────

function AccountRow(props: { account: Account }) {
  return (
    <div class="flex items-center gap-3 px-4 py-2.5 hover:bg-surface-hover transition-colors">
      <CreditCard size={16} class="text-text-tertiary shrink-0 ml-6" />
      <div class="flex-1 min-w-0">
        <p class="text-sm font-medium text-text truncate">
          {props.account.name}
        </p>
        <p class="text-xs text-text-tertiary capitalize">
          {props.account.type}
        </p>
      </div>
      <span class="text-sm font-medium text-text tabular-nums">
        {formatCurrency(props.account.balance_cache, props.account.currency)}
      </span>
    </div>
  );
}

// ── Helpers ──────────────────────────────────────────────────

function formatCurrency(amount: string | number, currency: string): string {
  const num =
    typeof amount === "string" ? Number.parseFloat(amount) : amount;
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
    }).format(num);
  } catch {
    return `${currency} ${num.toFixed(2)}`;
  }
}

function EmptyState(props: {
  title: string;
  description: string;
  onAction: () => void;
}) {
  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <Building2 size={48} class="text-text-tertiary mb-4" />
      <h2 class="text-lg font-semibold text-text">{props.title}</h2>
      <p class="text-sm text-text-secondary mt-1 max-w-xs">
        {props.description}
      </p>
      <Button variant="primary" size="sm" class="mt-4" onClick={props.onAction}>
        <Plus size={16} />
        Add Bank
      </Button>
    </div>
  );
}
