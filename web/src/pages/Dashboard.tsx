/**
 * Dashboard page — overview with summary cards and quick stats.
 */

import { useI18n } from "~/i18n";

export default function DashboardPage() {
  const t = useI18n();

  return (
    <div class="space-y-6">
      <h1 class="text-2xl font-bold text-text">
        {t("common.nav.dashboard") ?? "Dashboard"}
      </h1>

      {/* Placeholder cards — will be wired to real data in Phase 3+ */}
      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        <SummaryCard label="Net Worth" value="—" />
        <SummaryCard label="Income (month)" value="—" />
        <SummaryCard label="Expenses (month)" value="—" />
        <SummaryCard label="Savings Rate" value="—" />
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-6">
          <h2 class="text-sm font-medium text-text-secondary mb-4">
            Recent Transactions
          </h2>
          <p class="text-sm text-text-tertiary">No transactions yet.</p>
        </div>
        <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-6">
          <h2 class="text-sm font-medium text-text-secondary mb-4">
            Spending by Category
          </h2>
          <p class="text-sm text-text-tertiary">No data available.</p>
        </div>
      </div>
    </div>
  );
}

function SummaryCard(props: { label: string; value: string }) {
  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <p class="text-xs font-medium text-text-secondary uppercase tracking-wide">
        {props.label}
      </p>
      <p class="text-2xl font-semibold text-text mt-1">{props.value}</p>
    </div>
  );
}
