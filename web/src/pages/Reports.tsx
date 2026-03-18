/**
 * Reports page — tabbed analytics: Income/Expense, Category Trend,
 * Balance History, and Cash Flow with forecast.
 */

import {
  createEffect,
  createResource,
  createSignal,
  For,
  onCleanup,
  Show,
  Suspense,
} from "solid-js";
import { Download } from "lucide-solid";
import { useI18n } from "~/i18n";
import { api } from "~/api";
import type {
  IncomeExpenseReport,
  CategoryTrendReport,
  BalanceHistoryReport,
  CashFlowReport,
  Category,
} from "~/api";
import { Skeleton } from "~/components/ui";
import { initChart, CHART_COLORS } from "~/lib/chart";
import { formatCurrency } from "~/lib/format";

// ── CSV export helper ────────────────────────────────────────

/**
 * Serialises an array of row arrays into a CSV string and triggers a download.
 * Values containing commas or quotes are wrapped in double-quotes.
 */
function downloadCsv(filename: string, headers: string[], rows: (string | number | boolean)[][]): void {
  const escape = (v: string | number | boolean) => {
    const s = String(v);
    return s.includes(",") || s.includes('"') || s.includes("\n")
      ? `"${s.replace(/"/g, '""')}"`
      : s;
  };
  const lines = [
    headers.map(escape).join(","),
    ...rows.map((row) => row.map(escape).join(",")),
  ];
  const blob = new Blob([lines.join("\r\n")], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function ExportCsvButton(props: { onClick: () => void }) {
  const t = useI18n();
  return (
    <button
      type="button"
      onClick={props.onClick}
      class="flex items-center gap-1.5 h-8 rounded-[var(--radius)] border border-border px-3 text-xs font-medium text-text-secondary hover:bg-surface-hover hover:text-text transition-colors"
    >
      <Download size={13} />
      {t("reports.export.csv") ?? "Export CSV"}
    </button>
  );
}

// ── Date helpers ─────────────────────────────────────────────

function isoDate(d: Date): string {
  return d.toISOString().slice(0, 10);
}

function defaultRange(months = 12): { from: string; to: string } {
  const to = new Date();
  const from = new Date(to);
  from.setMonth(from.getMonth() - (months - 1));
  from.setDate(1);
  return { from: isoDate(from), to: isoDate(to) };
}

// ── Tab types ────────────────────────────────────────────────

type TabId = "income-expense" | "categories" | "balance-history" | "cash-flow";

// ── Page ─────────────────────────────────────────────────────

export default function ReportsPage() {
  const t = useI18n();
  const [activeTab, setActiveTab] = createSignal<TabId>("income-expense");

  const tabs: { id: TabId; label: string }[] = [
    { id: "income-expense", label: t("reports.tabs.incomeExpense") ?? "Income & Expense" },
    { id: "categories", label: t("reports.tabs.categories") ?? "Categories" },
    { id: "balance-history", label: t("reports.tabs.balanceHistory") ?? "Balance History" },
    { id: "cash-flow", label: t("reports.tabs.cashFlow") ?? "Cash Flow" },
  ];

  return (
    <div class="space-y-6">
      <h1 class="text-2xl font-bold text-text">
        {t("reports.title") ?? "Reports"}
      </h1>

      {/* Tab bar */}
      <div class="flex gap-1 border-b border-border">
        <For each={tabs}>
          {(tab) => (
            <button
              type="button"
              onClick={() => setActiveTab(tab.id)}
              class="px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors"
              classList={{
                "border-accent text-accent": activeTab() === tab.id,
                "border-transparent text-text-secondary hover:text-text": activeTab() !== tab.id,
              }}
            >
              {tab.label}
            </button>
          )}
        </For>
      </div>

      {/* Tab panels */}
      <Show when={activeTab() === "income-expense"}>
        <IncomeExpenseTab />
      </Show>
      <Show when={activeTab() === "categories"}>
        <CategoryTrendTab />
      </Show>
      <Show when={activeTab() === "balance-history"}>
        <BalanceHistoryTab />
      </Show>
      <Show when={activeTab() === "cash-flow"}>
        <CashFlowTab />
      </Show>
    </div>
  );
}

// ── Date range picker ────────────────────────────────────────

function DateRangePicker(props: {
  from: string;
  to: string;
  onChange: (from: string, to: string) => void;
}) {
  const t = useI18n();
  const [localFrom, setLocalFrom] = createSignal(props.from);
  const [localTo, setLocalTo] = createSignal(props.to);

  const presets = [
    { label: t("reports.dateRange.presets.last3months") ?? "Last 3 months", months: 3 },
    { label: t("reports.dateRange.presets.last6months") ?? "Last 6 months", months: 6 },
    { label: t("reports.dateRange.presets.last12months") ?? "Last 12 months", months: 12 },
  ];

  return (
    <div class="flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <label class="text-xs text-text-secondary">
          {t("reports.dateRange.from") ?? "From"}
        </label>
        <input
          type="date"
          value={localFrom()}
          onInput={(e) => setLocalFrom(e.currentTarget.value)}
          class="h-8 rounded-[var(--radius)] border border-border bg-surface px-2 text-sm"
        />
        <label class="text-xs text-text-secondary">
          {t("reports.dateRange.to") ?? "To"}
        </label>
        <input
          type="date"
          value={localTo()}
          onInput={(e) => setLocalTo(e.currentTarget.value)}
          class="h-8 rounded-[var(--radius)] border border-border bg-surface px-2 text-sm"
        />
        <button
          type="button"
          onClick={() => props.onChange(localFrom(), localTo())}
          class="h-8 rounded-[var(--radius)] bg-accent px-3 text-sm font-medium text-accent-foreground hover:bg-accent/90 transition-colors"
        >
          {t("reports.dateRange.apply") ?? "Apply"}
        </button>
      </div>
      <div class="flex gap-1">
        <For each={presets}>
          {(preset) => (
            <button
              type="button"
              onClick={() => {
                const r = defaultRange(preset.months);
                setLocalFrom(r.from);
                setLocalTo(r.to);
                props.onChange(r.from, r.to);
              }}
              class="rounded-full border border-border px-2.5 py-0.5 text-xs text-text-secondary hover:bg-surface-hover transition-colors"
            >
              {preset.label}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

// ── Income & Expense tab ─────────────────────────────────────

function IncomeExpenseTab() {
  const t = useI18n();
  const init = defaultRange(12);
  const [range, setRange] = createSignal(init);

  const [data] = createResource(range, (r) =>
    api.fetchIncomeExpenseReport(r.from, r.to),
  );

  const handleExport = () => {
    const report = data();
    if (!report) return;
    downloadCsv(
      `income-expense-${range().from}-${range().to}.csv`,
      ["Month", "Income", "Expenses", "Net"],
      report.months.map((m) => [
        m.month.slice(0, 7),
        m.income,
        m.expenses,
        String(parseFloat(m.income) - parseFloat(m.expenses)),
      ]),
    );
  };

  return (
    <div class="space-y-4">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <DateRangePicker
          from={range().from}
          to={range().to}
          onChange={(from, to) => setRange({ from, to })}
        />
        <Show when={data()}>
          <ExportCsvButton onClick={handleExport} />
        </Show>
      </div>
      <Suspense fallback={<Skeleton class="h-72" variant="rect" />}>
        <Show when={data.error}>
          <p class="text-sm text-danger">{t("reports.error") ?? "Failed to load."}</p>
        </Show>
        <Show when={data()}>
          {(report) => <IncomeExpenseChart data={report()} />}
        </Show>
      </Suspense>
    </div>
  );
}

function IncomeExpenseChart(props: { data: IncomeExpenseReport }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const months = props.data.months;
    if (!months.length) return;

    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    chart.setOption({
      tooltip: { trigger: "axis", axisPointer: { type: "shadow" } },
      legend: { data: ["Income", "Expenses", "Net"], bottom: 0 },
      grid: { left: 16, right: 16, top: 16, bottom: 48, containLabel: true },
      xAxis: {
        type: "category",
        data: months.map((m) => m.month.slice(0, 7)),
        axisLabel: { rotate: 30, fontSize: 11 },
      },
      yAxis: { type: "value", axisLabel: { fontSize: 11 } },
      series: [
        {
          name: "Income",
          type: "bar",
          data: months.map((m) => parseFloat(m.income)),
          itemStyle: { color: CHART_COLORS.income },
        },
        {
          name: "Expenses",
          type: "bar",
          data: months.map((m) => parseFloat(m.expenses)),
          itemStyle: { color: CHART_COLORS.expenses },
        },
        {
          name: "Net",
          type: "line",
          data: months.map((m) => parseFloat(m.income) - parseFloat(m.expenses)),
          itemStyle: { color: CHART_COLORS.net },
          lineStyle: { width: 2 },
          symbolSize: 5,
        },
      ],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <div ref={container} class="h-72 w-full" />
    </div>
  );
}

// ── Category trend tab ───────────────────────────────────────

async function fetchCategories(): Promise<Category[]> {
  const res = await api.fetchList<Category>("/api/categories");
  return res.data;
}

function CategoryTrendTab() {
  const t = useI18n();
  const init = defaultRange(12);
  const [range, setRange] = createSignal(init);
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [cats] = createResource(fetchCategories);

  const trendKey = () => {
    const id = selectedId();
    if (!id) return null;
    return { id, ...range() };
  };

  const [data] = createResource(trendKey, (k) =>
    k ? api.fetchCategoryTrend(k.id, k.from, k.to) : Promise.resolve(null),
  );

  const handleExport = () => {
    const report = data();
    if (!report) return;
    downloadCsv(
      `category-trend-${range().from}-${range().to}.csv`,
      ["Period", "Total", "Average"],
      report.periods.map((p) => [p.period.slice(0, 7), p.total, report.average]),
    );
  };

  return (
    <div class="space-y-4">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="flex flex-wrap items-center gap-3">
          <select
            class="h-8 rounded-[var(--radius)] border border-border bg-surface px-2 text-sm"
            onChange={(e) => setSelectedId(e.currentTarget.value || null)}
          >
            <option value="">
              {t("reports.categories.selectCategory") ?? "Select a category…"}
            </option>
            <For each={cats()}>
              {(cat) => <option value={cat.id}>{cat.name}</option>}
            </For>
          </select>
          <DateRangePicker
            from={range().from}
            to={range().to}
            onChange={(from, to) => setRange({ from, to })}
          />
        </div>
        <Show when={data()}>
          <ExportCsvButton onClick={handleExport} />
        </Show>
      </div>
      <Suspense fallback={<Skeleton class="h-72" variant="rect" />}>
        <Show when={data()}>
          {(report) => <CategoryTrendChart data={report()!} />}
        </Show>
        <Show when={!selectedId()}>
          <p class="text-sm text-text-secondary">
            {t("reports.categories.selectCategory") ?? "Select a category above."}
          </p>
        </Show>
      </Suspense>
    </div>
  );
}

function CategoryTrendChart(props: { data: CategoryTrendReport }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const periods = props.data.periods;
    if (!periods.length) return;

    const avg = parseFloat(props.data.average);
    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    chart.setOption({
      tooltip: { trigger: "axis" },
      legend: { data: ["Spend", "Average"], bottom: 0 },
      grid: { left: 16, right: 16, top: 16, bottom: 48, containLabel: true },
      xAxis: {
        type: "category",
        data: periods.map((p) => p.period.slice(0, 7)),
        axisLabel: { rotate: 30, fontSize: 11 },
      },
      yAxis: { type: "value" },
      series: [
        {
          name: "Spend",
          type: "bar",
          data: periods.map((p) => parseFloat(p.total)),
          itemStyle: { color: CHART_COLORS.expenses },
        },
        {
          name: "Average",
          type: "line",
          data: periods.map(() => avg),
          lineStyle: { type: "dashed", color: CHART_COLORS.net },
          itemStyle: { color: CHART_COLORS.net },
          symbol: "none",
        },
      ],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <div ref={container} class="h-72 w-full" />
    </div>
  );
}

// ── Balance history tab ──────────────────────────────────────

function BalanceHistoryTab() {
  const t = useI18n();
  const init = defaultRange(12);
  const [range, setRange] = createSignal(init);

  const [data] = createResource(range, (r) =>
    api.fetchBalanceHistory(r.from, r.to),
  );

  const handleExport = () => {
    const report = data();
    if (!report) return;
    const accountHeaders = report.accounts.map((a) => a.name);
    downloadCsv(
      `balance-history-${range().from}-${range().to}.csv`,
      ["Date", ...accountHeaders, "Net Worth"],
      report.snapshots.map((s) => [
        s.date,
        ...report.accounts.map((a) => {
          const b = s.balances.find((b) => b.account_id === a.id);
          return b?.balance ?? "0";
        }),
        s.net_worth,
      ]),
    );
  };

  return (
    <div class="space-y-4">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <DateRangePicker
          from={range().from}
          to={range().to}
          onChange={(from, to) => setRange({ from, to })}
        />
        <Show when={data()}>
          <ExportCsvButton onClick={handleExport} />
        </Show>
      </div>
      <Suspense fallback={<Skeleton class="h-72" variant="rect" />}>
        <Show when={data.error}>
          <p class="text-sm text-danger">{t("reports.error") ?? "Failed to load."}</p>
        </Show>
        <Show when={data()}>
          {(report) => <BalanceHistoryChart data={report()} />}
        </Show>
      </Suspense>
    </div>
  );
}

function BalanceHistoryChart(props: { data: BalanceHistoryReport }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const { snapshots, accounts } = props.data;
    if (!snapshots.length) return;

    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    const dates = snapshots.map((s) => s.date);

    // One series per account + net worth
    const accountSeries = accounts.map((acc, i) => ({
      name: acc.name,
      type: "line" as const,
      smooth: true,
      data: snapshots.map((s) => {
        const bal = s.balances.find((b) => b.account_id === acc.id);
        return parseFloat(bal?.balance ?? "0");
      }),
      itemStyle: { color: CHART_COLORS.palette[i % CHART_COLORS.palette.length] },
      symbol: "none",
    }));

    const netSeries = {
      name: "Net Worth",
      type: "line" as const,
      smooth: true,
      data: snapshots.map((s) => parseFloat(s.net_worth)),
      itemStyle: { color: CHART_COLORS.net },
      lineStyle: { width: 2.5 },
      symbol: "none",
    };

    chart.setOption({
      tooltip: { trigger: "axis" },
      legend: {
        data: [...accounts.map((a) => a.name), "Net Worth"],
        bottom: 0,
        textStyle: { fontSize: 11 },
      },
      grid: { left: 16, right: 16, top: 16, bottom: 48, containLabel: true },
      xAxis: {
        type: "category",
        data: dates,
        axisLabel: { rotate: 30, fontSize: 11 },
      },
      yAxis: { type: "value" },
      series: [...accountSeries, netSeries],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
      <div ref={container} class="h-72 w-full" />
    </div>
  );
}

// ── Cash flow tab ────────────────────────────────────────────

function CashFlowTab() {
  const t = useI18n();
  const init = defaultRange(6);
  const [range, setRange] = createSignal(init);

  const [data] = createResource(range, (r) =>
    api.fetchCashFlowReport(r.from, r.to),
  );

  const handleExport = () => {
    const report = data();
    if (!report) return;
    const allPeriods = [...report.periods, ...report.forecast];
    downloadCsv(
      `cash-flow-${range().from}-${range().to}.csv`,
      ["Period", "Income", "Expenses", "Net", "Forecast"],
      allPeriods.map((p) => [
        p.period.slice(0, 7),
        p.income,
        p.expenses,
        p.net,
        p.is_forecast ? "yes" : "no",
      ]),
    );
  };

  return (
    <div class="space-y-4">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <DateRangePicker
          from={range().from}
          to={range().to}
          onChange={(from, to) => setRange({ from, to })}
        />
        <Show when={data()}>
          <ExportCsvButton onClick={handleExport} />
        </Show>
      </div>
      <Suspense fallback={<Skeleton class="h-72" variant="rect" />}>
        <Show when={data.error}>
          <p class="text-sm text-danger">{t("reports.error") ?? "Failed to load."}</p>
        </Show>
        <Show when={data()}>
          {(report) => <CashFlowChart data={report()} />}
        </Show>
      </Suspense>
    </div>
  );
}

function CashFlowChart(props: { data: CashFlowReport }) {
  let container!: HTMLDivElement;

  createEffect(async () => {
    const { periods, forecast } = props.data;
    const allPeriods = [...periods, ...forecast];
    if (!allPeriods.length) return;

    const chart = await initChart(container);
    onCleanup(() => chart.dispose());

    const labels = allPeriods.map((p) => p.period.slice(0, 7));
    const incomeSeries = allPeriods.map((p) => ({
      value: parseFloat(p.income),
      itemStyle: p.is_forecast
        ? { color: CHART_COLORS.income, opacity: 0.45 }
        : { color: CHART_COLORS.income },
    }));
    const expSeries = allPeriods.map((p) => ({
      value: parseFloat(p.expenses),
      itemStyle: p.is_forecast
        ? { color: CHART_COLORS.expenses, opacity: 0.45 }
        : { color: CHART_COLORS.expenses },
    }));
    const netSeries = allPeriods.map((p) => ({
      value: parseFloat(p.net),
      itemStyle: p.is_forecast
        ? { color: CHART_COLORS.forecast, opacity: 0.55 }
        : { color: parseFloat(p.net) >= 0 ? CHART_COLORS.net : CHART_COLORS.expenses },
    }));

    chart.setOption({
      tooltip: { trigger: "axis", axisPointer: { type: "cross" } },
      legend: { data: ["Income", "Expenses", "Net"], bottom: 0 },
      grid: { left: 16, right: 16, top: 16, bottom: 48, containLabel: true },
      xAxis: {
        type: "category",
        data: labels,
        axisLabel: { rotate: 30, fontSize: 11 },
      },
      yAxis: { type: "value" },
      series: [
        { name: "Income", type: "bar", stack: "total", data: incomeSeries },
        { name: "Expenses", type: "bar", stack: "exp", data: expSeries },
        {
          name: "Net",
          type: "line",
          data: netSeries,
          lineStyle: { width: 2 },
          symbolSize: 5,
        },
      ],
    });

    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(container);
    onCleanup(() => observer.disconnect());
  });

  return (
    <div class="space-y-2">
      {/* Summary row */}
      <div class="flex gap-4">
        <div class="rounded-[var(--radius)] border border-border bg-surface px-3 py-1.5 text-xs">
          <span class="text-text-secondary">
            {props.data.avg_income ? `Avg income: ${formatCurrency(props.data.avg_income, "USD")}` : ""}
          </span>
        </div>
        <div class="rounded-[var(--radius)] border border-border bg-surface px-3 py-1.5 text-xs">
          <span class="text-text-secondary">
            {props.data.avg_expenses ? `Avg expenses: ${formatCurrency(props.data.avg_expenses, "USD")}` : ""}
          </span>
        </div>
      </div>
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface p-4">
        <div ref={container} class="h-72 w-full" />
      </div>
    </div>
  );
}
