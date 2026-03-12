# QIF

QIF (Quicken Interchange Format) is a legacy format created by Intuit for Quicken. Many banks and financial tools still export in QIF.

## Auto-Detection

QIF files are identified by a first line starting with `!Type:` (e.g. `!Type:Bank`, `!Type:CCard`).

Accepted extensions: `.qif`

## Format Overview

QIF uses line-prefix encoding — each field starts with a single character:

| Prefix | Field |
|--------|-------|
| `D` | Date |
| `T` | Amount |
| `P` | Payee |
| `M` | Memo |
| `N` | Check number |
| `L` | Category |
| `^` | End-of-record separator |

### Example

```
!Type:Bank
D01/15/2025
T-42.50
PSupermarket
MGrocery shopping
LFood:Groceries
^
D01/14/2025
T3200.00
PEmployer Inc
MSalary January
LIncome:Salary
^
```

## Extracted Fields

| Field | Source |
|-------|--------|
| Date | `D` line |
| Amount | `T` line |
| Description | `P` (payee) line, with `M` (memo) as notes |

## No Column Mapping Needed

QIF has a fixed line-prefix structure, so the **Configure** step is skipped.

## Tips

- **Date format** — QIF dates vary by region (`MM/DD/YYYY` in the US, `DD/MM/YYYY` in Europe). RustVault's date parser handles common variants, but you can override in the configure step if needed.
- **Categories** — QIF `L` lines contain Quicken category names. RustVault imports these as the description note; actual categorization should be done via [auto-rules](../features/auto-rules.md).
