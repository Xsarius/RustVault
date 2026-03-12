# Quick Tour

This guide walks through the main screens of RustVault so you can get
productive quickly.  If you haven't installed yet, start with the
[Installation](installation.md) page first.

---

## 1. Login & Registration

Open `http://localhost:3000` (or your server's URL).  New users click
**Create account** to register with an email and password.  If your
instance has OIDC/SSO configured you will also see a "Sign in with …"
button.

After logging in, the app redirects to the **Dashboard**.

## 2. Dashboard

The Dashboard is your financial overview at a glance:

| Widget | What it shows |
|--------|---------------|
| **Net Worth** | Sum of all account balances |
| **Monthly Income / Expenses** | Current-month totals with a small spark chart |
| **Recent Transactions** | The latest 5–10 entries across all accounts |
| **Budget Progress** | How much of each budget envelope has been spent |

Click any widget to jump to its detail page.

## 3. Banks & Accounts

Navigate via the sidebar (**Banks & Accounts**).

- **Add a bank** — click **+ Add Bank**, enter the name and (optional)
  logo URL.
- **Add an account** — expand a bank, click **+ Add Account**, choose
  the account type (Checking, Savings, Credit Card, …), currency, and
  starting balance.
- **Edit / Delete** — use the ⋯ menu on each card.

Accounts are grouped by bank and show their current balance with
income/expense colour coding.

## 4. Categories

RustVault uses a two-level category tree (parent → child) with icons
and colours.

- Navigate to **Categories** in the sidebar.
- Click **+ Add Category** to create a top-level category or assign a
  parent to nest it.
- Drag-and-drop (or use the ⋯ menu) to reorder.
- Categories are used for budgets, rules, and reports.

## 5. Tags

Tags are lightweight labels you can attach to any transaction (e.g.
"vacation", "tax-deductible").

- Navigate to **Tags**.
- Click **+ Add Tag** → enter the name and pick an optional colour.
- Tags appear as coloured chips on the transaction list and can be
  filtered in Reports.

## 6. Transactions *(coming soon)*

The Transactions page will let you browse, search, and bulk-edit
entries.  For now, transactions are best viewed from the Dashboard's
"Recent Transactions" widget.

## 7. Import *(coming soon)*

The Import Wizard will guide you through mapping CSV, MT940, OFX, and
other formats.  See [Your First Import](first-import.md) for a preview.

## 8. Budget & Reports *(coming soon)*

Placeholder pages are in place.  Budget envelopes and interactive charts
will be built in Phase 3.

## 9. Settings

Open **Settings** from the sidebar (or the user menu in the top bar).
Three tabs are available:

| Tab | Contents |
|-----|----------|
| **General** | Display name, default currency, date format |
| **Security** | Change password, manage API keys |
| **Appearance** | Theme (light / dark / system), language selector |

## 10. Theme Toggle

Click the sun/moon icon in the top bar to switch between **light**,
**dark**, and **system** themes.  Your preference is saved in
`localStorage` and survives page reloads.

---

> **Tip:** All pages are lazy-loaded, so the first visit to each route
> shows a brief skeleton loader — subsequent visits are instant.
