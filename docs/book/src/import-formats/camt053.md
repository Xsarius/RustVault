# CAMT.053

CAMT.053 (Cash Management — Bank to Customer Statement) is an ISO 20022 XML format widely used by European banks for account statements.

## Auto-Detection

CAMT.053 files are identified by the presence of a `<Document` element with a namespace containing `camt.053`.

Accepted extensions: `.xml`, `.camt053`, `.camt`

## Format Overview

CAMT.053 is a structured XML format:

```xml
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <Stmt>
      <Ntry>             <!-- one per transaction -->
        <BookgDt>...</BookgDt>
        <Amt Ccy="EUR">42.50</Amt>
        <CdtDbtInd>DBIT</CdtDbtInd>
        <NtryDtls>
          <TxDtls>
            <RmtInf>
              <Ustrd>Supermarket</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
```

## Extracted Fields

| Field | XML path |
|-------|----------|
| Date | `Ntry > BookgDt` or `Ntry > ValDt` |
| Amount | `Ntry > Amt` with `CdtDbtInd` for sign |
| Currency | `Ntry > Amt @Ccy` attribute |
| Description | `Ntry > NtryDtls > TxDtls > RmtInf > Ustrd` |

## No Column Mapping Needed

CAMT.053's XML schema defines all fields, so the **Configure** step is skipped. Upload and review.

## Tips

- **Multiple statements** — a single file can contain multiple `<Stmt>` blocks for different accounts. All are parsed.
- **Version tolerance** — RustVault handles multiple CAMT.053 schema versions (001.02 through 001.08).
