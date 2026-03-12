# OFX / QFX

OFX (Open Financial Exchange) is a data format for exchanging financial data. QFX (Quicken Financial Exchange) is Intuit's variant of OFX.

## Auto-Detection

OFX files are identified by the presence of `OFXHEADER` (SGML variant) or `<OFX>` (XML variant) in the file content.

Accepted extensions: `.ofx`, `.qfx`

## Format Overview

OFX comes in two flavours:

| Version | Format | Identification |
|---------|--------|---------------|
| OFX 1.x | SGML (no closing tags) | Starts with `OFXHEADER:100` |
| OFX 2.x | XML | Starts with `<?OFX` or contains `<OFX>` |

Transactions are contained in `<STMTTRN>` elements within a `<BANKTRANLIST>` block.

## Extracted Fields

| Field | OFX element |
|-------|-------------|
| Date | `<DTPOSTED>` |
| Amount | `<TRNAMT>` |
| Description | `<NAME>` and/or `<MEMO>` |
| Type | `<TRNTYPE>` (DEBIT, CREDIT, etc.) |
| Reference | `<FITID>` (financial institution transaction ID) |

## No Column Mapping Needed

OFX has a well-defined schema, so the **Configure** step is skipped. Upload → Preview → Confirm.

## Tips

- **SGML parsing** — older OFX 1.x files lack closing tags (e.g. `<NAME>Grocery Store` instead of `<NAME>Grocery Store</NAME>`). RustVault handles both variants.
- **FITID deduplication** — the `<FITID>` field is used for duplicate detection across imports from the same institution.
