# Transactions

The Transactions page is the core of RustVault — where you browse, search, filter, and manage your financial data.

## Viewing Transactions

Transactions are displayed in an infinite-scroll table with cursor-based pagination, designed to handle tens of thousands of entries without lag.

Each row shows:

| Column | Description |
|--------|-------------|
| Date | Transaction date |
| Description | Payee or memo text |
| Amount | Colour-coded: green for income, red for expenses |
| Category | Assigned category (click to change inline) |
| Account | Source account |
| Tags | Coloured chips (click to add/remove) |
| Status | Reviewed / unreviewed indicator |

## Searching

A debounced search bar filters transactions by description, payee, and notes. Results update as you type.

## Filtering

The filter panel supports:

- **Date range** — custom range or presets (this month, last 30 days, this year, etc.)
- **Account** — one or more accounts
- **Category** — one or more categories (includes children)
- **Tag** — one or more tags
- **Type** — income, expense, or transfer
- **Status** — reviewed, unreviewed, or all
- **Amount range** — minimum and/or maximum

Filters can be combined and are reflected in the URL for bookmarking.

## Creating Transactions

Click **+ Add Transaction** to manually create an entry. Required fields: date, amount, description, and account. Optional: category, tags, notes, payee.

## Editing

- **Inline editing** — click a category or tag cell to change it directly in the table.
- **Detail view** — click a transaction row to open the full edit form with all fields.

## Bulk Actions

Select multiple transactions using checkboxes, then choose an action:

| Action | Description |
|--------|-------------|
| **Categorize** | Assign a category to all selected transactions |
| **Tag** | Add or remove tags from the selection |
| **Mark reviewed** | Mark selected as reviewed |
| **Delete** | Permanently delete selected transactions |

## Transaction Types

| Type | Description |
|------|-------------|
| `income` | Money received (salary, refunds) |
| `expense` | Money spent (bills, purchases) |
| `transfer` | Movement between your own accounts |

Transfers link two transactions — one debit and one credit — between different accounts.

## Reviewed Status

Transactions start as **unreviewed** after import. Marking a transaction as reviewed indicates you've verified it. This is useful for reconciliation workflows — filter to unreviewed transactions to see what still needs attention.
