# Budgeting

Budgets in RustVault let you plan spending for any date range, track how your actual transactions compare to your plan, and quickly copy or compare budgets between periods.

## Concepts

| Term | Meaning |
|------|---------|
| **Budget** | A spending plan for a given period (e.g. "March 2026") with a reporting currency. |
| **Budget line** | A single category allocation within a budget (e.g. €300 for Groceries). |
| **Actual amount** | The sum of matching transactions automatically computed from your imported data. |
| **Summary** | The rolled-up view showing planned vs. actual per line and in total. |
| **Recurring budget** | A budget marked to repeat on a schedule using an iCal RRULE. |

---

## Creating a Budget

1. Navigate to **Budget** in the sidebar.
2. Click **New Budget**.
3. Fill in the form:
   - **Name** — a human-readable label, e.g. "March 2026".
   - **Period Start / End** — the inclusive date range the budget covers.
   - **Currency** — the ISO 4217 code used for all planned amounts (e.g. `EUR`, `USD`).
   - **Recurring** — enable this to mark the budget as repeating. Provide an RRULE string (e.g. `FREQ=MONTHLY;INTERVAL=1`) to define the recurrence pattern.
   - **Notes** — any optional free-text notes.
4. Click **Create**.

---

## Adding Budget Lines

After creating a budget, click on it to open the detail view, then switch to the **Budget Lines** tab.

1. Click **Add Category**.
2. Select a category from the dropdown.
3. Enter the **Planned Amount** — how much you intend to spend in this category.
4. Optionally add a note.
5. Click **Add**.

Repeat for each category you want to track. You can edit or remove lines at any time.

---

## Viewing the Budget Overview

Click any budget in the list to open its detail view. The **Overview** tab shows:

- **KPI cards** — total planned, total actual, and remaining balance, all formatted in your locale's currency style.
- **Overall progress bar** — colour-coded green (on track), amber (≥ 80% used), or red (over budget).
- **Per-category breakdown** — a mini progress bar and "X used / Y remaining" line for every budget line.
- **Refresh Actuals** — click this to re-query your imported transactions and update the cached actual amounts.

---

## Comparing Two Budgets

In the detail view, switch to the **Comparison** tab:

1. A row of buttons lists all your other budgets.
2. Click one to load it as the comparison target.
3. A table appears with columns for *this* budget's planned/actual and the *other* budget's planned/actual, per category.

This is useful for spotting spending trends across months.

---

## Copying a Budget

To quickly start a new budget period based on an existing one:

1. In the budget list, click the **Copy** icon on any budget card.
2. Enter a name for the new budget.
3. Set the new period start and end dates.
4. Click **Copy**.

All budget lines (categories + planned amounts) are duplicated into the new budget. Actual amounts start at zero and will be computed from transactions in the new period.

---

## Recurring Budgets

Mark a budget as **Recurring** and supply an iCal RRULE (e.g. `FREQ=MONTHLY;INTERVAL=1`). The recurrence rule is stored with the budget and can be used to automatically generate the next period's budget shell.

> **Note:** Automatic generation of the next period is planned for a future release. You can use the **Copy** feature in the meantime.

---

## Exchange Rates

RustVault fetches daily exchange rates from the [ECB XML feed](https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml). Rates are stored in the database and used to convert transaction amounts when your budget currency differs from an account's currency.

To manually refresh rates, make a `POST` request to `/api/exchange-rates/refresh`. The response includes the number of rates updated.

---

## API Reference

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/budgets` | List budgets. Add `?include_archived=true` to include archived budgets. |
| `POST` | `/api/budgets` | Create a budget. |
| `GET` | `/api/budgets/:id` | Get a single budget. |
| `PUT` | `/api/budgets/:id` | Update budget metadata. |
| `DELETE` | `/api/budgets/:id` | Delete a budget (cascades to lines). |
| `GET` | `/api/budgets/:id/summary` | Get planned vs. actual summary. |
| `POST` | `/api/budgets/:id/copy` | Copy lines to a new period. |
| `GET` | `/api/budgets/:id/lines` | List budget lines. |
| `POST` | `/api/budgets/:id/lines` | Add a budget line. |
| `POST` | `/api/budgets/:id/lines/bulk` | Replace all lines at once. |
| `PUT` | `/api/budgets/:id/lines/:line_id` | Update a line. |
| `DELETE` | `/api/budgets/:id/lines/:line_id` | Remove a line. |
| `GET` | `/api/exchange-rates` | List latest exchange rates. |
| `POST` | `/api/exchange-rates/refresh` | Fetch and store current ECB rates. |
