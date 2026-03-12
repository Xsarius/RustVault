# JSON

RustVault can import transactions from JSON files — useful for exports from other finance apps or custom scripts.

## Auto-Detection

JSON files are identified by content starting with `[` (array) or `{` (object).

Accepted extensions: `.json`

## Expected Structure

The file should contain an array of transaction objects:

```json
[
  {
    "date": "2025-01-15",
    "amount": -42.50,
    "description": "Supermarket",
    "currency": "EUR",
    "notes": "Weekly groceries"
  },
  {
    "date": "2025-01-14",
    "amount": 3200.00,
    "description": "Salary",
    "currency": "EUR"
  }
]
```

## Column Mapping

JSON field names vary between sources, so you must map fields during the **Configure** step:

| RustVault field | Required | Notes |
|-----------------|----------|-------|
| Date | Yes | String or ISO 8601 formatted |
| Amount | Yes | Number — negative for expenses |
| Description | Yes | Transaction description |
| Currency | No | ISO 4217 code |
| Notes | No | Additional metadata |

## Tips

- **Nested objects** — RustVault reads top-level array entries. If your data is nested inside a wrapper object (e.g. `{"transactions": [...]}`), extract the array before importing.
- **Encoding** — UTF-8 is required.
