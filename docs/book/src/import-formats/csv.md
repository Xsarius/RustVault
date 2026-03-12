# CSV

CSV (Comma-Separated Values) is the most common bank export format. RustVault supports arbitrary CSV layouts through column mapping.

## Auto-Detection

CSV is detected as a fallback when no other format matches. RustVault's heuristic checks the first 5 lines for consistent delimiter counts (comma `,`, semicolon `;`, or tab `\t`).

Accepted extensions: `.csv`

## Column Mapping

Since every bank uses a different CSV layout, you must map columns to RustVault fields during the **Configure** step of the import wizard:

| RustVault field | Required | Notes |
|-----------------|----------|-------|
| Date | Yes | Transaction date — specify the format (e.g. `DD.MM.YYYY`, `YYYY-MM-DD`) |
| Amount | Yes | Negative values = expenses. Some banks use separate debit/credit columns. |
| Description | Yes | Payee, memo, or description text |
| Currency | No | ISO 4217 code. Defaults to the account currency if omitted. |
| Notes | No | Additional notes or reference numbers |

## Common Banks

### Single-amount column

```csv
Date,Description,Amount,Balance
2025-01-15,Supermarket,-42.50,1234.56
2025-01-14,Salary,3200.00,1277.06
```

Map: Date → `Date`, Description → `Description`, Amount → `Amount`.

### Separate debit/credit columns

```csv
Date;Payee;Debit;Credit;Balance
15.01.2025;Supermarket;42,50;;1234,56
14.01.2025;Salary;;3200,00;1277,06
```

Map: Date → `Date`, Payee → `Description`, Debit → `Amount (debit)`, Credit → `Amount (credit)`.

## Tips

- **Delimiter** — RustVault auto-detects the delimiter. European banks often use semicolons.
- **Decimal separator** — commas as decimal separators (e.g. `42,50`) are handled automatically.
- **Encoding** — UTF-8 is expected. If your bank exports in ISO-8859-1 or Windows-1252, convert the file first.
- **Header row** — the first row is assumed to be a header. If your file has no header, the wizard lets you assign column names manually.
