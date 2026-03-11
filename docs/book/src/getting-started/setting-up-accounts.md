# Setting Up Accounts

After [installing RustVault](installation.md) and registering your user, the next step is to create one or more **banks** and **accounts** to organise your finances.

## Concepts

| Term | Description |
|------|-------------|
| **Bank** | A financial institution (e.g. "ING", "Revolut", "Cash"). RustVault uses "bank" loosely — it can represent any source of funds. |
| **Account** | A single account within a bank (e.g. "Main Checking", "Savings", "Credit Card"). Each account has a currency and type. |

### Account Types

| Type | Use Case |
|------|----------|
| `checking` | Day-to-day current / transaction accounts |
| `savings` | Savings or deposit accounts |
| `credit` | Credit card accounts |
| `investment` | Brokerage / investment accounts |
| `loan` | Loan or mortgage accounts |

## Step 1 — Create a Bank

Use the API or the web UI to create a bank that represents your financial institution.

**API example:**

```bash
curl -X POST http://localhost:8080/api/banks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "ING"}'
```

Response:

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "ING",
    "is_archived": false,
    "created_at": "2026-03-15T10:00:00Z",
    "updated_at": "2026-03-15T10:00:00Z"
  }
}
```

## Step 2 — Create an Account

With a bank in place, create one or more accounts under it.

```bash
curl -X POST http://localhost:8080/api/accounts \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "bank_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Main Checking",
    "currency": "EUR",
    "account_type": "checking"
  }'
```

Repeat for each account you want to track. A typical setup might look like:

| Bank | Account | Currency | Type |
|------|---------|----------|------|
| ING | Main Checking | EUR | checking |
| ING | Savings Plus | EUR | savings |
| Revolut | Daily Spending | EUR | checking |
| Revolut | USD Account | USD | checking |

## Step 3 — Set Up Categories (Optional)

RustVault ships with no default categories — you create the ones that fit your workflow. Categories are hierarchical: a parent category can have children.

```bash
# Create a parent category
curl -X POST http://localhost:8080/api/categories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Food & Drinks", "category_type": "expense"}'

# Bulk-create children
curl -X POST http://localhost:8080/api/categories/bulk \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "categories": [
      {"name": "Groceries", "parent_id": "<parent-id>", "category_type": "expense"},
      {"name": "Restaurants", "parent_id": "<parent-id>", "category_type": "expense"},
      {"name": "Coffee", "parent_id": "<parent-id>", "category_type": "expense"}
    ]
  }'
```

### Category Types

| Type | Description |
|------|-------------|
| `income` | Money coming in (salary, refunds, etc.) |
| `expense` | Money going out (bills, subscriptions, etc.) |
| `transfer` | Movements between your own accounts |

## Step 4 — Create Tags (Optional)

Tags provide a second axis of organisation alongside categories. Typical tags: `work`, `vacation`, `recurring`, `tax-deductible`.

```bash
curl -X POST http://localhost:8080/api/tags/bulk \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tags": [
      {"name": "recurring", "color": "#4CAF50"},
      {"name": "vacation", "color": "#FF9800"},
      {"name": "tax-deductible", "color": "#2196F3"}
    ]
  }'
```

## What's Next?

Once your banks, accounts, and categories are set up you can start recording transactions — either manually or by importing bank statements. See the **Import Pipeline** documentation for supported formats.
