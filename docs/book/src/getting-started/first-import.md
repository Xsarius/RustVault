# Your First Import

This guide walks you through importing your first bank statement into RustVault using the **Import Wizard**.

## Prerequisites

- RustVault is running and you are logged in
- At least one [bank and account](setting-up-accounts.md) has been created
- You have a bank statement file (CSV, OFX, MT940, etc.)

## Step 1 — Open the Import Wizard

Navigate to **Import** in the sidebar. The wizard has four steps: Upload → Configure → Preview → Confirm.

## Step 2 — Upload Your File

Drag-and-drop your bank statement onto the upload area, or click **Browse** to select a file.

RustVault auto-detects the file format using magic bytes, content inspection, and file extension. Supported formats:

| Format | Extensions |
|--------|-----------|
| CSV | `.csv` |
| MT940 | `.mt940`, `.sta` |
| OFX / QFX | `.ofx`, `.qfx` |
| QIF | `.qif` |
| CAMT.053 | `.xml`, `.camt053`, `.camt` |
| Spreadsheet | `.xlsx`, `.xls`, `.ods` |
| JSON | `.json` |

## Step 3 — Configure Column Mapping

For **structured formats** (OFX, MT940, CAMT.053, QIF) this step is skipped — the parser knows the field layout.

For **CSV, JSON, and spreadsheet** files you need to map columns to RustVault fields:

| RustVault field | Description | Required |
|-----------------|-------------|----------|
| Date | Transaction date | Yes |
| Amount | Transaction amount | Yes |
| Description | Payee / description text | Yes |
| Currency | ISO 4217 code | No (defaults to account currency) |
| Notes | Additional notes | No |

Select the target account for the import and choose the correct date format if auto-detection didn't get it right.

## Step 4 — Preview

RustVault parses the file and shows a table of the transactions it found. Review the list to check:

- **Dates** are parsed correctly
- **Amounts** have the right sign (negative = expense, positive = income)
- **Duplicates** are highlighted — transactions that already exist in the account
- **Auto-categorization** — any matching rules are applied and categories shown

You can edit individual transactions before importing.

## Step 5 — Confirm

Click **Import** to execute. RustVault inserts all transactions in a single database transaction — if anything fails, nothing is saved.

After a successful import you'll see a summary:

- Total transactions imported
- Duplicates skipped
- Rules applied
- New categories created (if any)

## Tips

- **Start small** — import one month first to verify the mapping is correct before importing a full history.
- **Create rules first** — set up a few [auto-categorization rules](../features/auto-rules.md) before importing so transactions are categorized automatically.
- **Check your date format** — European banks often use `DD.MM.YYYY` while US banks use `MM/DD/YYYY`. The configure step lets you specify the exact format.

## What's Next?

- [Import Pipeline](../features/import-pipeline.md) — learn about format detection and the full processing flow
- [Auto-Categorization Rules](../features/auto-rules.md) — automate category assignment
- [Import Formats](../import-formats/csv.md) — format-specific guidance
