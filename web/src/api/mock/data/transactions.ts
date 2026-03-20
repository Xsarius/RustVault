/**
 * Demo mode seed data — Transactions.
 *
 * ~60 transactions spread over Oct 2025–Mar 2026 across the demo accounts.
 * A mix of salary, rent, groceries, dining, streaming, transfer, etc.
 */

import type { Transaction } from "~/api/types";

function tx(
  id: string,
  accountId: string,
  categoryId: string | null,
  type: Transaction["transaction_type"],
  amount: string,
  date: string,
  description: string,
  payee: string | null = null,
  tagIds: string[] = [],
  isReviewed = true,
): Transaction {
  return {
    id,
    user_id: "demo-user",
    account_id: accountId,
    category_id: categoryId,
    import_id: null,
    transaction_type: type,
    amount,
    currency: "EUR",
    date,
    description,
    original_desc: null,
    payee,
    reference: null,
    notes: null,
    is_reviewed: isReviewed,
    is_deleted: false,
    is_duplicate: false,
    metadata: {},
    tag_ids: tagIds,
    created_at: `${date}T12:00:00Z`,
    updated_at: `${date}T12:00:00Z`,
  };
}

// Account IDs (matching data/accounts.ts)
const REV = "acc-revolut-eur";
const LUN = "acc-lunar-dkk";
const ZEN = "acc-zen-usd";
const SAV = "acc-revolut-savings";
const CAR = "acc-lunar-credit";

export const DEMO_TRANSACTIONS: Transaction[] = [
  // ── October 2025 ─────────────────────────────────────────────
  tx("txn-001", REV, "cat-salary", "income", "4200.00", "2025-10-01", "October Salary", "Employer GmbH", ["tag-recurring"]),
  tx("txn-002", REV, "cat-rent", "expense", "1200.00", "2025-10-02", "Rent October", "Estate Agency", ["tag-recurring"]),
  tx("txn-003", REV, "cat-groceries", "expense", "87.45", "2025-10-05", "Lidl weekly shop", "Lidl"),
  tx("txn-004", REV, "cat-streaming", "expense", "15.99", "2025-10-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-005", REV, "cat-dining", "expense", "34.20", "2025-10-10", "Dinner at The Oak", "The Oak Restaurant"),
  tx("txn-006", REV, "cat-transport", "expense", "2.90", "2025-10-11", "Metro ticket", "Transport Co."),
  tx("txn-007", REV, "cat-groceries", "expense", "63.10", "2025-10-14", "ALDI groceries", "ALDI"),
  tx("txn-008", REV, "cat-gym", "expense", "39.99", "2025-10-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-009", REV, "cat-streaming", "expense", "9.99", "2025-10-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-010", REV, "cat-dining", "expense", "22.50", "2025-10-18", "Lunch with colleagues", null),
  tx("txn-011", REV, "cat-groceries", "expense", "54.80", "2025-10-21", "Carrefour market run", "Carrefour"),
  tx("txn-012", REV, "cat-health", "expense", "18.00", "2025-10-24", "Pharmacy — vitamins", "Dr. Green Pharmacy"),
  tx("txn-013", SAV, null, "transfer", "300.00", "2025-10-28", "Transfer to savings", null, ["tag-recurring"]),
  tx("txn-014", REV, "cat-dining", "expense", "45.60", "2025-10-30", "Weekend brunch", null),
  tx("txn-015", REV, "cat-shopping", "expense", "79.90", "2025-10-31", "H&M — winter jacket", "H&M"),

  // ── November 2025 ────────────────────────────────────────────
  tx("txn-016", REV, "cat-salary", "income", "4200.00", "2025-11-01", "November Salary", "Employer GmbH", ["tag-recurring"]),
  tx("txn-017", REV, "cat-rent", "expense", "1200.00", "2025-11-03", "Rent November", "Estate Agency", ["tag-recurring"]),
  tx("txn-018", REV, "cat-groceries", "expense", "91.30", "2025-11-06", "Lidl + organic market", "Lidl"),
  tx("txn-019", REV, "cat-streaming", "expense", "15.99", "2025-11-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-020", REV, "cat-dining", "expense", "28.75", "2025-11-12", "Thai restaurant", null),
  tx("txn-021", REV, "cat-transport", "expense", "55.00", "2025-11-14", "Train ticket Berlin", "Deutsche Bahn", ["tag-business"]),
  tx("txn-022", REV, "cat-groceries", "expense", "48.20", "2025-11-17", "ALDI weekly", "ALDI"),
  tx("txn-023", REV, "cat-gym", "expense", "39.99", "2025-11-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-024", REV, "cat-streaming", "expense", "9.99", "2025-11-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-025", REV, "cat-entertain", "expense", "32.00", "2025-11-20", "Cinema tickets x2", "Vue Cinema"),
  tx("txn-026", SAV, null, "transfer", "300.00", "2025-11-28", "Transfer to savings", null, ["tag-recurring"]),
  tx("txn-027", REV, "cat-shopping", "expense", "129.00", "2025-11-29", "Black Friday — headphones", "Amazon", ["tag-online"]),

  // ── December 2025 ────────────────────────────────────────────
  tx("txn-028", REV, "cat-salary", "income", "5400.00", "2025-12-01", "December Salary + bonus", "Employer GmbH", ["tag-recurring"]),
  tx("txn-029", REV, "cat-rent", "expense", "1200.00", "2025-12-02", "Rent December", "Estate Agency", ["tag-recurring"]),
  tx("txn-030", REV, "cat-groceries", "expense", "165.40", "2025-12-21", "Christmas shopping groceries", "Carrefour"),
  tx("txn-031", REV, "cat-dining", "expense", "210.00", "2025-12-24", "Christmas Eve dinner", "La Maison Restaurant"),
  tx("txn-032", REV, "cat-shopping", "expense", "340.00", "2025-12-10", "Christmas gifts", null, ["tag-shared"]),
  tx("txn-033", REV, "cat-streaming", "expense", "15.99", "2025-12-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-034", REV, "cat-streaming", "expense", "9.99", "2025-12-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-035", REV, "cat-gym", "expense", "39.99", "2025-12-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-036", REV, "cat-travel", "expense", "389.00", "2025-12-26", "Flights — New Year trip", "Ryanair", ["tag-vacation"]),
  tx("txn-037", REV, "cat-travel", "expense", "180.00", "2025-12-27", "Hotel — 2 nights", "Booking.com", ["tag-vacation"]),
  tx("txn-038", SAV, null, "transfer", "500.00", "2025-12-28", "End-of-year savings top-up", null, ["tag-recurring"]),

  // ── January 2026 ─────────────────────────────────────────────
  tx("txn-039", REV, "cat-salary", "income", "4500.00", "2026-01-01", "January Salary", "Employer GmbH", ["tag-recurring"]),
  tx("txn-040", REV, "cat-rent", "expense", "1200.00", "2026-01-02", "Rent January", "Estate Agency", ["tag-recurring"]),
  tx("txn-041", REV, "cat-groceries", "expense", "78.90", "2026-01-05", "Lidl weekly", "Lidl"),
  tx("txn-042", REV, "cat-streaming", "expense", "15.99", "2026-01-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-043", REV, "cat-streaming", "expense", "9.99", "2026-01-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-044", REV, "cat-gym", "expense", "39.99", "2026-01-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-045", REV, "cat-health", "expense", "65.00", "2026-01-18", "Dentist appointment", "City Dental Clinic"),
  tx("txn-046", REV, "cat-groceries", "expense", "82.10", "2026-01-20", "ALDI + bakery", "ALDI"),
  tx("txn-047", REV, "cat-dining", "expense", "41.00", "2026-01-22", "Sushi evening", null),
  tx("txn-048", SAV, null, "transfer", "300.00", "2026-01-28", "Transfer to savings", null, ["tag-recurring"]),
  tx("txn-049", REV, null, "expense", "136.50", "2026-01-31", "Utility bill — electricity", "Vattenfall", [], false),

  // ── February 2026 ────────────────────────────────────────────
  tx("txn-050", REV, "cat-salary", "income", "4500.00", "2026-02-01", "February Salary", "Employer GmbH", ["tag-recurring"]),
  tx("txn-051", REV, "cat-rent", "expense", "1200.00", "2026-02-03", "Rent February", "Estate Agency", ["tag-recurring"]),
  tx("txn-052", REV, "cat-groceries", "expense", "95.20", "2026-02-07", "Lidl + market", "Lidl"),
  tx("txn-053", REV, "cat-streaming", "expense", "15.99", "2026-02-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-054", REV, "cat-dining", "expense", "68.00", "2026-02-14", "Valentine's dinner", null),
  tx("txn-055", REV, "cat-gym", "expense", "39.99", "2026-02-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-056", REV, "cat-streaming", "expense", "9.99", "2026-02-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-057", REV, "cat-shopping", "expense", "210.00", "2026-02-20", "New running shoes", "Nike Store"),
  tx("txn-058", SAV, null, "transfer", "300.00", "2026-02-28", "Transfer to savings", null, ["tag-recurring"]),
  tx("txn-059", REV, "cat-health", "expense", "170.00", "2026-02-26", "Glasses — new prescription", "Specsavers", [], false),

  // ── March 2026 ───────────────────────────────────────────────
  tx("txn-060", REV, "cat-salary", "income", "4500.00", "2026-03-01", "March Salary", "Employer GmbH", ["tag-recurring"]),
  tx("txn-061", REV, "cat-rent", "expense", "1200.00", "2026-03-02", "Rent March", "Estate Agency", ["tag-recurring"]),
  tx("txn-062", REV, "cat-groceries", "expense", "73.40", "2026-03-06", "ALDI weekly", "ALDI"),
  tx("txn-063", REV, "cat-streaming", "expense", "15.99", "2026-03-08", "Netflix subscription", "Netflix", ["tag-recurring"]),
  tx("txn-064", REV, "cat-dining", "expense", "38.50", "2026-03-11", "Pasta night out", null),
  tx("txn-065", REV, "cat-gym", "expense", "39.99", "2026-03-15", "FitnessPark membership", "FitnessPark", ["tag-recurring"]),
  tx("txn-066", REV, "cat-streaming", "expense", "9.99", "2026-03-16", "Spotify Premium", "Spotify", ["tag-recurring"]),
  tx("txn-067", REV, "cat-groceries", "expense", "68.20", "2026-03-19", "Carrefour + deli", "Carrefour"),
  tx("txn-068", REV, null, "expense", "41.80", "2026-03-21", "Miscellaneous expense", null, [], false),
  tx("txn-069", REV, "cat-transport", "expense", "12.00", "2026-03-24", "Taxi home", "Bolt"),
  tx("txn-070", SAV, null, "transfer", "300.00", "2026-03-28", "Transfer to savings", null, ["tag-recurring"]),

  // ── LUN / ZEN / CAR for variety ──────────────────────────────
  tx("txn-071", LUN, "cat-groceries", "expense", "320.00", "2025-11-10", "Netto grocery run (DKK)", "Netto"),
  tx("txn-072", ZEN, "cat-shopping", "expense", "49.99", "2026-01-15", "USD online purchase", "Amazon US", ["tag-online"]),
  tx("txn-073", CAR, "cat-dining", "expense", "85.00", "2026-02-12", "Business dinner (card)", null, ["tag-business"]),
  tx("txn-074", REV, "cat-freelance", "income", "750.00", "2026-03-10", "Freelance invoice #12", "Client XYZ"),
];
