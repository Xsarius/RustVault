/**
 * Dashboard page — live summary cards and visualisation charts.
 *
 * Fetches the dashboard summary from `GET /api/reports/summary` and renders:
 *   - 4 stat cards: net worth, month income, month expenses, savings rate
 *   - Monthly income/expense bar chart (last 12 months)
 *   - Spending by category donut/pie chart (current month)
 *   - Unreviewed transaction badge
 */

import {
  createResource,
  createEffect,
  onCleanup,
  Show,
  Suspense,
  For,
  createMemo,
} from "solid-js";
import { A } from "@solidjs/router";
import { AlertCircle, HelpCircle, CreditCard, PiggyBank, TrendingUp, Building2 } from "lucide-solid";
import { DashboardSkeleton, Tooltip } from "~/components/ui";
import { api } from "~/api";
import type { DashboardSummary, Bank, Account, Budget, BudgetSummary, Category } from "~/api";
import { useI18n, useLocale } from "~/i18n";
import { formatCurrency } from "~/lib/format";

// ── Data fetching ────────────────────────────────────────────

async function loadSummary(): Promise<DashboardSummary> {
  return api.fetchDashboardSummary();
}

interface DashboardExtras {
  banks: Bank[];
  accounts: Account[];
  currentBudget: Budget | null;
  currentBudgetSummary: BudgetSummary | null;
  categories: Category[];
}

async function fetchDashboardExtras(): Promise<DashboardExtras> {
  const today = new Date().toISOString().slice(0, 10);

  const [banksRes, accountsRes, budgets, categoriesRes] = await Promise.all([
    api.fetchList<Bank>("/api/banks"),
    api.fetchList<Account>("/api/accounts"),
    api.listBudgets(false),
    api.fetchList<Category>("/api/categories"),
  ]);

  const currentBudget =
    budgets.find((b) => b.period_start <= today && b.period_end >= today) ?? null;

  const currentBudgetSummary = currentBudget
    ? await api.getBudgetSummary(currentBudget.id)
    : null;

  return {
    banks: banksRes.data.filter((b) => !b.is_archived),
    accounts: accountsRes.data.filter((a) => !a.is_archived),
    currentBudget,
    currentBudgetSummary,
    categories: categoriesRes.data,
  };
}

// ── Page ─────────────────────────────────────────────────────

export default function DashboardPage() {
  const t = useI18n();
  const [summary] = createResource(loadSummary);
  const [extras] = createResource(fetchDashboardExtras);

  return (
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("common.nav.dashboard") ?? "Dashboard"}
        </h1>
        <Show when={(summary()?.unreviewed_count ?? 0) > 0}>
          <A
            href="/transactions?is_reviewed=false"
            class="flex items-center gap-1.5 rounded-full bg-warning/10 px-3 py-1 text-xs font-medium text-warning hover:bg-warning/20 transition-colors"
          >
            <AlertCircle size={14} />
            {summary()!.unreviewed_count}{" "}
            {t("reports.summary.unreviewedSuffix") ?? "need review"}
          </A>
        </Show>
      </div>

      <Suspense fallback={<DashboardSkeleton />}>
        <Show when={summary.error}>
          <ErrorCard message={String(summary.error)} />
        </Show>

        <Show when={summary()}>
          {(data) => <DashboardContent data={data()} extras={extras() ?? null} />}
        </Show>
      </Suspense>
    </div>
  );
}

// ── Dashboard content ────────────────────────────────────────

function DashboardContent(props: { data: DashboardSummary; extras: DashboardExtras | null }) {
  const t = useI18n();
  const { locale } = useLocale();
  const d = () => props.data;

  // Derive vs-last-month changes from monthly_trend
  const trend = () => d().monthly_trend;
  const lastMonth = () => {
    const tr = trend();
    return tr.length >= 2 ? tr[tr.length - 2] : null;
  };

  const incomeChange = createMemo(() => {
    const lm = lastMonth();
    if (!lm) return null;
    const prev = parseFloat(lm.income);
    if (!prev) return null;
    return ((parseFloat(d().month_income) - prev) / prev) * 100;
  });

  const expensesChange = createMemo(() => {
    const lm = lastMonth();
    if (!lm) return null;
    const prev = parseFloat(lm.expenses);
    if (!prev) return null;
    return ((parseFloat(d().month_expenses) - prev) / prev) * 100;
  });

  const savingsRateTrend = createMemo(() =>
    trend()
      .filter((p) => parseFloat(p.income) > 0)
      .map((p) => {
        const inc = parseFloat(p.income);
        const exp = parseFloat(p.expenses);
        return ((inc - exp) / inc) * 100;
      }),
  );

  const lastMonthSavingsRate = createMemo(() => {
    const lm = lastMonth();
    if (!lm) return null;
    const inc = parseFloat(lm.income);
    if (!inc) return null;
    return ((inc - parseFloat(lm.expenses)) / inc) * 100;
  });

  const savingsLabel = () => {
    const rate = d().savings_rate;
    if (rate === null) return "—";
    return `${rate.toFixed(1)}%`;
  };

  const extras = () => props.extras;

  return (
    <div class="space-y-6">
      {/* Net Worth hero */}
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
        <p class="text-xs font-medium text-text-secondary uppercase tracking-wide">
          {t("reports.summary.netWorth") ?? "Net Worth"}
        </p>
        <p class="text-3xl font-bold text-text mt-1">
          {formatCurrency(d().net_worth, "USD")}
        </p>
        <Show when={lastMonth()}>
          {(() => {
            const netChange = parseFloat(d().month_income) - parseFloat(d().month_expenses);
            const positive = netChange >= 0;
            return (
              <p class={`text-sm mt-0.5 ${positive ? "text-success" : "text-danger"}`}>
                {positive ? "↑" : "↓"} {formatCurrency(Math.abs(netChange), "USD")} this month
              </p>
            );
          })()}
        </Show>
      </div>

      {/* Income + Expenses */}
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <StatCard
          label={t("reports.summary.monthIncome") ?? "Income (month)"}
          value={formatCurrency(d().month_income, "USD")}
          positive
          help={t("reports.help.monthIncome") ?? ""}
          change={incomeChange()}
          changePositive={true}
        />
        <StatCard
          label={t("reports.summary.monthExpenses") ?? "Expenses (month)"}
          value={formatCurrency(d().month_expenses, "USD")}
          help={t("reports.help.monthExpenses") ?? ""}
          change={expensesChange()}
          changePositive={false}
        />
      </div>

      {/* Accounts */}
      <Show when={(extras()?.banks.length ?? 0) > 0}>
        <AccountsSection
          banks={extras()!.banks}
          accounts={extras()!.accounts}
          locale={locale()}
        />
      </Show>

      {/* This month budget */}
      <Show when={extras()?.currentBudgetSummary}>
        <BudgetThisMonthSection
          summary={extras()!.currentBudgetSummary!}
          categories={extras()!.categories}
          currency={extras()!.currentBudget!.currency}
          locale={locale()}
        />
      </Show>

      {/* Savings Rate */}
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
        <p class="text-xs font-medium text-text-secondary uppercase tracking-wide">
          {t("reports.summary.savingsRate") ?? "Savings Rate"}
        </p>
        <div class="flex items-end justify-between mt-1">
          <div>
            <p
              class={`text-3xl font-bold ${d().savings_rate !== null && d().savings_rate! >= 0 ? "text-success" : "text-danger"}`}
            >
              {savingsLabel()}
            </p>
            <Show when={lastMonthSavingsRate() !== null}>
              {(() => {
                const prev = lastMonthSavingsRate()!;
                const curr = d().savings_rate ?? 0;
                const positive = curr >= prev;
                return (
                  <p class={`text-sm mt-0.5 ${positive ? "text-success" : "text-danger"}`}>
                    {positive ? "↑" : "↓"} from {prev.toFixed(1)}% last month
                  </p>
                );
              })()}
            </Show>
          </div>
          <SavingsSparkline values={savingsRateTrend()} />
        </div>
      </div>
    </div>
  );
}

// ── Accounts section ─────────────────────────────────────────

function AccountsSection(props: { banks: Bank[]; accounts: Account[]; locale: string }) {
  const t = useI18n();

  const grouped = createMemo(() =>
    props.banks
      .map((bank) => ({
        bank,
        accounts: props.accounts
          .filter((a) => a.bank_id === bank.id)
          .sort((a, b) => a.sort_order - b.sort_order),
      }))
      .filter((g) => g.accounts.length > 0),
  );

  const accountIcon = (type: Account["type"]) => {
    switch (type) {
      case "savings": return <PiggyBank size={13} class="shrink-0" />;
      case "investment": return <TrendingUp size={13} class="shrink-0" />;
      case "loan": return <Building2 size={13} class="shrink-0" />;
      default: return <CreditCard size={13} class="shrink-0" />;
    }
  };

  const fmt = (amount: string, currency: string) =>
    new Intl.NumberFormat(props.locale, {
      style: "currency",
      currency,
      maximumFractionDigits: 2,
    }).format(parseFloat(amount));

  return (
    <div>
      <p class="text-xs font-medium text-text-secondary uppercase tracking-wide mb-2">
        {t("common.nav.banks") ?? "Accounts"}
      </p>
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface divide-y divide-border">
        <For each={grouped()}>
          {({ bank, accounts }) => {
            const initials = bank.name.slice(0, 1).toUpperCase();
            return (
              <div class="px-4 py-3">
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-6 h-6 rounded-full bg-primary/15 flex items-center justify-center shrink-0">
                    <span class="text-xs font-bold text-primary">{initials}</span>
                  </div>
                  <span class="font-medium text-text text-sm">{bank.name}</span>
                </div>
                <div class="space-y-1 pl-8">
                  <For each={accounts}>
                    {(acc) => {
                      const negative = parseFloat(acc.balance_cache) < 0;
                      const isCreditOrLoan = acc.type === "credit" || acc.type === "loan";
                      return (
                        <div class="flex items-center justify-between gap-2 text-sm">
                          <div class="flex items-center gap-1.5 text-text-secondary min-w-0">
                            {accountIcon(acc.type)}
                            <span class="truncate">{acc.name}</span>
                            <Show when={isCreditOrLoan}>
                              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-danger/10 text-danger shrink-0">
                                {acc.type === "loan" ? (t("accounts.type.loan") ?? "Loan") : (t("accounts.type.credit") ?? "Credit")}
                              </span>
                            </Show>
                          </div>
                          <span class={`font-mono text-sm shrink-0 ${negative ? "text-danger" : "text-text"}`}>
                            {fmt(acc.balance_cache, acc.currency)}
                          </span>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
}

// ── Budget this month section ─────────────────────────────────

function BudgetThisMonthSection(props: {
  summary: BudgetSummary;
  categories: Category[];
  currency: string;
  locale: string;
}) {
  const totalPlanned = () => parseFloat(props.summary.total_planned_expenses || "0");
  const totalActual = () => parseFloat(props.summary.total_actual_expenses || "0");
  const overallPct = () =>
    totalPlanned() > 0 ? Math.min(100, (totalActual() / totalPlanned()) * 100) : 0;

  const fmt = (amount: string | number) =>
    new Intl.NumberFormat(props.locale, {
      style: "currency",
      currency: props.currency,
      maximumFractionDigits: 0,
    }).format(typeof amount === "string" ? parseFloat(amount) : amount);

  const barColor = (pct: number) => {
    if (pct >= 100) return "bg-danger";
    if (pct >= 80) return "bg-amber-400";
    return "bg-emerald-500";
  };

  const categoryName = (id: string | null) =>
    id ? (props.categories.find((c) => c.id === id)?.name ?? id) : "—";

  const expenseLines = () =>
    props.summary.lines.filter((l) => parseFloat(l.planned_amount) > 0);

  return (
    <div>
      <p class="text-xs font-medium text-text-secondary uppercase tracking-wide mb-2">
        This Month
      </p>
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4 space-y-4">
        <div>
          <div class="h-2.5 rounded-full bg-surface-hover overflow-hidden mb-1.5">
            <div
              class={`h-full rounded-full transition-all ${barColor(overallPct())}`}
              style={{ width: `${Math.min(overallPct(), 100)}%` }}
            />
          </div>
          <p class="text-xs text-text-secondary">
            {fmt(totalActual())} of {fmt(totalPlanned())} budget — {overallPct().toFixed(0)}% used
          </p>
        </div>

        <div class="space-y-2.5">
          <For each={expenseLines()}>
            {(line) => {
              const pct = parseFloat(line.percent_used) || 0;
              const over = pct >= 100;
              return (
                <div>
                  <div class="flex items-center justify-between text-sm mb-1">
                    <span class="font-medium text-text">
                      {categoryName(line.category_id)}
                      <Show when={over}>
                        <span class="ml-1.5 text-danger text-xs">⚠</span>
                      </Show>
                    </span>
                    <span class={`font-mono text-xs ${over ? "text-danger" : "text-text-secondary"}`}>
                      {fmt(line.actual_amount)} / {fmt(line.planned_amount)}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="flex-1 h-1.5 rounded-full bg-surface-hover overflow-hidden">
                      <div
                        class={`h-full rounded-full ${barColor(pct)}`}
                        style={{ width: `${Math.min(pct, 100)}%` }}
                      />
                    </div>
                    <span class={`text-xs font-mono w-9 text-right ${over ? "text-danger" : "text-text-secondary"}`}>
                      {pct.toFixed(0)}%
                    </span>
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
}

// ── Savings sparkline (SVG) ───────────────────────────────────

function SavingsSparkline(props: { values: number[] }) {
  if (props.values.length < 2) return null as unknown as JSX.Element;
  const W = 80;
  const H = 32;
  const vals = props.values;
  const min = Math.min(...vals);
  const max = Math.max(...vals);
  const range = max - min || 1;
  const pts = vals
    .map((v, i) => {
      const x = (i / (vals.length - 1)) * W;
      const y = H - ((v - min) / range) * (H - 4) - 2;
      return `${x},${y}`;
    })
    .join(" ");
  return (
    <svg width={W} height={H} class="opacity-70 shrink-0">
      <polyline
        points={pts}
        fill="none"
        stroke="#22c55e"
        stroke-width="1.5"
        stroke-linejoin="round"
        stroke-linecap="round"
      />
    </svg>
  );
}

// ── Helper components ────────────────────────────────────────

function StatCard(props: {
  label: string;
  value: string;
  positive?: boolean;
  help?: string;
  change?: number | null;
  changePositive?: boolean;
}) {
  const changeSign = () => {
    if (props.change == null) return null;
    return props.change >= 0 ? "↑" : "↓";
  };
  const changeColor = () => {
    if (props.change == null) return "";
    const isGood = props.changePositive ? props.change >= 0 : props.change <= 0;
    return isGood ? "text-success" : "text-danger";
  };
  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <div class="flex items-center gap-1.5">
        <p class="text-xs font-medium text-text-secondary uppercase tracking-wide">
          {props.label}
        </p>
        <Show when={props.help}>
          <Tooltip content={props.help!}>
            <HelpCircle size={12} class="text-text-tertiary cursor-help flex-shrink-0" />
          </Tooltip>
        </Show>
      </div>
      <p
        class="text-2xl font-semibold mt-1"
        classList={{
          "text-text": props.positive === undefined,
          "text-success": props.positive === true,
          "text-danger": props.positive === false,
        }}
      >
        {props.value}
      </p>
      <Show when={props.change != null}>
        <p class={`text-xs mt-0.5 ${changeColor()}`}>
          {changeSign()} {Math.abs(props.change!).toFixed(1)}% vs last month
        </p>
      </Show>
    </div>
  );
}

function ErrorCard(props: { message: string }) {
  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-6 text-danger text-sm">
      {props.message}
    </div>
  );
}
