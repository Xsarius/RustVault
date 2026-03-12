# Import Pipeline

RustVault's import pipeline converts bank statement files into structured transactions through a multi-stage process.

## Pipeline Stages

```
Upload → Format Detection → Parsing → Mapping → Deduplication → Rules → Preview → Insert
```

### 1. Upload

The user uploads a file via the web UI or the API. The server validates:

- File size (configurable via `import.max_file_size`, default 50 MB)
- File extension (configurable via `import.allowed_extensions`)

### 2. Format Detection

RustVault auto-detects the file format using three strategies in order:

1. **Magic bytes** — ZIP archives (XLSX/ODS) start with `PK`; OLE2 files (XLS) start with `D0 CF 11 E0`
2. **Content inspection** — the first few lines are checked for format signatures:
   - OFX: `OFXHEADER` or `<OFX>`
   - CAMT.053: `<Document` with `camt.053` namespace
   - MT940: starts with `:20:` (SWIFT header)
   - QIF: starts with `!Type:`
   - JSON: starts with `[` or `{`
3. **Extension fallback** — if content inspection is inconclusive, the file extension is used

### 3. Parsing

Each format has a dedicated parser that extracts raw transaction records:

| Parser | Crate module | Field extraction |
|--------|-------------|-----------------|
| CSV | `parsers::csv` | Column mapping via user configuration |
| MT940 | `parsers::mt940` | SWIFT tag fields (`:61:`, `:86:`) |
| OFX/QFX | `parsers::ofx` | SGML/XML `<STMTTRN>` elements |
| QIF | `parsers::qif` | Line-prefixed records (`D`, `T`, `P`, `M`) |
| CAMT.053 | `parsers::camt053` | XML `<Ntry>` elements |
| Spreadsheet | `parsers::spreadsheet` | Row-based with header mapping |
| JSON | `parsers::json` | Array-of-objects with field mapping |

A shared date parser (`parsers::date`) handles 20+ date format variants.

### 4. Column Mapping

Structured formats (OFX, MT940, CAMT.053, QIF) have fixed field layouts. For CSV, JSON, and spreadsheets, the user maps columns to RustVault fields (date, amount, description, currency, notes).

### 5. Deduplication

Before inserting, RustVault checks for existing transactions with matching date, amount, and description within the same account. Potential duplicates are flagged for the user to review.

### 6. Auto-Categorization

Any active [rules](auto-rules.md) are applied to the parsed transactions. Matched transactions receive their category and/or tags automatically.

### 7. Preview & Confirm

The user reviews the parsed transactions, makes adjustments, and confirms. The import is executed within a single database transaction — all-or-nothing.

## API

Import is also available via the REST API:

```bash
curl -X POST http://localhost:8080/api/import/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@statement.csv" \
  -F "account_id=<account-uuid>"
```

## Import History

Each import is recorded with a timestamp, file name, format, transaction count, and status. Navigate to **Import → History** to review past imports.
