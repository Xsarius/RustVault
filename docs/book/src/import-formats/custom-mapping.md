# Custom Mapping

For formats that require column mapping (CSV, JSON, and spreadsheets), RustVault's import wizard includes a mapping configuration step.

## When Is Mapping Needed?

| Format | Mapping required |
|--------|-----------------|
| CSV | Yes — always |
| JSON | Yes — always |
| XLSX / XLS / ODS | Yes — always |
| MT940 | No — fixed structure |
| OFX / QFX | No — fixed structure |
| QIF | No — fixed structure |
| CAMT.053 | No — fixed structure |

## Mapping Fields

During the **Configure** step, you assign source columns to RustVault fields:

### Required fields

- **Date** — the column containing the transaction date. You may also need to specify the date format (e.g. `DD.MM.YYYY`, `YYYY-MM-DD`, `MM/DD/YYYY`).
- **Amount** — the column with the transaction amount. If your file has separate debit and credit columns, map both individually.
- **Description** — the column with the payee or description text.

### Optional fields

- **Currency** — ISO 4217 currency code. Defaults to the account's currency.
- **Notes** — additional reference or memo text.

## Handling Special Layouts

### Separate debit / credit columns

Some banks export debits and credits in separate columns:

```csv
Date;Description;Debit;Credit
15.01.2025;Supermarket;42.50;
14.01.2025;Salary;;3200.00
```

Map **Debit** as the debit-amount column and **Credit** as the credit-amount column. RustVault combines them into a single signed amount.

### Amounts with currency symbols

If amounts include currency symbols or thousand separators (e.g. `€1.234,56`), RustVault strips them automatically before parsing.

### Inverted sign convention

Some banks export expenses as positive values. Use the **Invert amounts** toggle in the configure step to flip the sign.

## Saving Mappings

Column mappings are remembered per account and format combination. The next time you import a file with the same column layout, the mapping is pre-filled.
