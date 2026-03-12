# XLSX / XLS / ODS

RustVault can import transactions from spreadsheet files — Microsoft Excel (XLSX, XLS) and LibreOffice/OpenDocument (ODS).

## Auto-Detection

Spreadsheet formats are identified by magic bytes:

| Format | Detection |
|--------|-----------|
| XLSX / ODS | ZIP archive (starts with `PK` / `50 4B`) |
| XLS | OLE2 compound document (starts with `D0 CF 11 E0`) |

Accepted extensions: `.xlsx`, `.xls`, `.ods`

## Column Mapping

Spreadsheets have no standard layout, so you must map columns during the **Configure** step — the same workflow as [CSV](csv.md).

| RustVault field | Required | Notes |
|-----------------|----------|-------|
| Date | Yes | Supports both date-typed cells and text dates |
| Amount | Yes | Numeric cells or text with currency symbols |
| Description | Yes | Payee, memo, or description |
| Currency | No | Defaults to account currency |
| Notes | No | Additional reference text |

## Tips

- **Sheet selection** — if the file contains multiple sheets, you are prompted to choose which one to import.
- **Header row** — the first row is treated as column headers by default.
- **Merged cells** — avoid merged cells in your export; they can interfere with column detection.
- **Formulas** — RustVault reads computed cell values, not formulas.
