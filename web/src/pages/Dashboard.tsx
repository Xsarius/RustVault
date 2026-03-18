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
} from "solid-js";
import { A } from "@solidjs/router";
import { AlertCircle, HelpCircle } from "lucide-solid";
import { DashboardSkeleton, Tooltip } from "~/components/ui";
import { api } from "~/api";
import type { DashboardSummary } from "~/api";
import { useI18n } from "~/i18n";
import { formatCurrency } from "~/lib/format";
import { initChart, CHART_COLORS } from "~/lib/chart";

// ── Data fetching ────────────────────────────────────────────

async function loadSummary(): Promise<DashboardSummary> {
  return api.fetchDashboardSummary();
}

// ── Page ─────────────────────────────────────────────────────

export default function DashboardPage() {
  const t = useI18n();
  const [summary] = createResource(loadSummary);

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
          {(data) => <DashboardContent data={data()} />}
        </Show>
      </Suspense>
    </div>
  );
}

// ── Dashboard content ────────────────────────────────────────

function DashboardContent(props: { data: DashboardSummary }) {
  const t = useI18n();
  const d = () => props.data;

  const savingsLabel = () => {
    const rate = d().savings_rate;
    if (rate === null) return "—";
    return `${rate.toFixed(1)}%`;
  };

  return (
    <div class="space-y-6">
      {/* Stat cards */}
      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        <StatCard
          label={t("reports.summary.netWorth") ?? "Net Worth"}
          value={formatCurrency(d().net_worth, "USD")}
          positive
          help={t("reports.help.netWorth") ?? ""}
        />
        <StatCard
          label={t("reports.summary.monthIncome") ?? "Income (month)"}
          value={formatCurrency(d().month_income, "USD")}
          positive
          help={t("reports.help.monthIncome") ?? ""}
        />
        <StatCard
          label={t("reports.summary.monthExpenses") ?? "Expenses (month)"}
          value={formatCurrency(d().month_expenses, "USD")}
          help={t("reports.help.monthExpenses") ?? ""}
        />
        <StatCard
          label={t("reports.summary.savingsRate") ?? "Savings Rate"}
          value={savingsLabel()}
          positive={d().savings_rate !== null && d().savings_rate! >= 0}
          help={t("reports.help.savingsRate") ?? ""}
        />
      </div>

      {/* Charts */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ChartCard title={t("reports.incomeExpense.title") ?? "Income vs Expenses"}>
          <MonthlyTrendChart data={props.data} />
        </ChartCard>

        <ChartCard title={t("reports.categories.title") ?? "Spending by Category"}>
          <CategoryDonutChart data={props.data} />
        </ChartCard>
      </div>
    </div>
  );
}

// ── Monthly trend bar chart ──────────────────────────────────

function MonthlyTrendChart(props: { data: DashboardSummary }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const points = props.data.monthly_trend;
    if (!points.length) return;

    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    chart.setOption({
      tooltip: {
        trigger: "axis",
        axisPointer: { type: "shadow" },
      },
      legend: { data: ["Income", "Expenses"], bottom: 0 },
      grid: { left: 16, right: 16, top: 16, bottom: 40, containLabel: true },
      xAxis: {
        type: "category",
        data: points.map((p) => p.month.slice(0, 7)), // YYYY-MM
        axisLabel: { rotate: 30, fontSize: 11 },
      },
      yAxis: { type: "value", axisLabel: { fontSize: 11 } },
      series: [
        {
          name: "Income",
          type: "bar",
          data: points.map((p) => parseFloat(p.income)),
          itemStyle: { color: CHART_COLORS.income },
        },
        {
          name: "Expenses",
          type: "bar",
          data: points.map((p) => parseFloat(p.expenses)),
          itemStyle: { color: CHART_COLORS.expenses },
        },
      ],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return <div ref={container} class="h-56 w-full" />;
}

// ── Spending donut chart ─────────────────────────────────────

function CategoryDonutChart(props: { data: DashboardSummary }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const categories = props.data.spending_by_category;
    if (!categories.length) return;

    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    chart.setOption({
      tooltip: { trigger: "item", formatter: "{b}: {d}%" },
      legend: { orient: "vertical", right: 8, top: "middle", textStyle: { fontSize: 11 } },
      series: [
        {
          type: "pie",
          radius: ["40%", "70%"],
          center: ["35%", "50%"],
          avoidLabelOverlap: false,
          label: { show: false },
          data: categories.map((c, i) => ({
            name: c.category_name ?? "Uncategorised",
            value: parseFloat(c.total),
            itemStyle: { color: CHART_COLORS.palette[i % CHART_COLORS.palette.length] },
          })),
        },
      ],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return <div ref={container} class="h-56 w-full" />;
}

// ── Helper components ────────────────────────────────────────

function StatCard(props: { label: string; value: string; positive?: boolean; help?: string }) {
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
    </div>
  );
}

function ChartCard(props: { title: string; children: any }) {
  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <h2 class="text-sm font-medium text-text-secondary mb-3">{props.title}</h2>
      {props.children}
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
