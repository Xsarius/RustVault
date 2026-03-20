/**
 * Demo mode — Transactions mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  Transaction,
  NewTransaction,
  UpdateTransaction,
  BulkUpdateTransactions,
  TransactionListQuery,
  PaginatedResponse,
  ApiResponse,
} from "~/api/types";

function fakeMeta(total: number) {
  return { page_size: total, has_more: false };
}

function applyFilters(
  txns: Transaction[],
  q: TransactionListQuery,
): Transaction[] {
  return txns.filter((t) => {
    if (q.account_id && t.account_id !== q.account_id) return false;
    if (q.category_id && t.category_id !== q.category_id) return false;
    if (q.transaction_type && t.transaction_type !== q.transaction_type) return false;
    if (q.date_from && t.date < q.date_from) return false;
    if (q.date_to && t.date > q.date_to) return false;
    if (q.is_reviewed !== undefined && t.is_reviewed !== q.is_reviewed) return false;
    if (q.tag_id && !t.tag_ids?.includes(q.tag_id)) return false;
    if (q.q) {
      const needle = q.q.toLowerCase();
      if (
        !t.description.toLowerCase().includes(needle) &&
        !(t.payee ?? "").toLowerCase().includes(needle)
      ) return false;
    }
    return true;
  });
}

export async function listTransactions(
  q: TransactionListQuery = {},
): Promise<PaginatedResponse<Transaction>> {
  const filtered = applyFilters([...demoStore.transactions], q).sort(
    (a, b) => b.date.localeCompare(a.date),
  );
  const limited = q.limit ? filtered.slice(0, q.limit) : filtered;
  return simulate({ data: limited, meta: fakeMeta(limited.length) });
}

export async function getTransaction(id: string): Promise<ApiResponse<Transaction>> {
  const t = demoStore.transactions.find((t) => t.id === id);
  if (!t) throw new Error("Transaction not found");
  return simulate({ data: t });
}

export async function createTransaction(
  body: NewTransaction,
): Promise<ApiResponse<Transaction>> {
  const txn: Transaction = {
    id: `txn-${crypto.randomUUID()}`,
    user_id: "demo-user",
    account_id: body.account_id,
    category_id: body.category_id ?? null,
    import_id: null,
    transaction_type: body.transaction_type,
    amount: body.amount,
    currency: "EUR",
    date: body.date,
    description: body.description,
    original_desc: null,
    payee: body.payee ?? null,
    reference: null,
    notes: body.notes ?? null,
    is_reviewed: true,
    is_deleted: false,
    is_duplicate: false,
    metadata: {},
    tag_ids: body.tag_ids ?? [],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("transactions", (prev) => [txn, ...prev]);
  return simulate({ data: txn });
}

export async function updateTransaction(
  id: string,
  body: UpdateTransaction,
): Promise<ApiResponse<Transaction>> {
  setDemoStore("transactions", (prev) =>
    prev.map((t) =>
      t.id === id ? { ...t, ...body, updated_at: new Date().toISOString() } : t,
    ),
  );
  const updated = demoStore.transactions.find((t) => t.id === id)!;
  return simulate({ data: updated });
}

export async function deleteTransaction(id: string): Promise<void> {
  setDemoStore("transactions", (prev) => prev.filter((t) => t.id !== id));
  return simulate(undefined);
}

export async function bulkUpdateTransactions(
  body: BulkUpdateTransactions,
): Promise<ApiResponse<{ updated: number }>> {
  const ids = new Set(body.transaction_ids);
  setDemoStore("transactions", (prev) =>
    prev.map((t) => {
      if (!ids.has(t.id)) return t;
      return {
        ...t,
        ...(body.category_id !== undefined ? { category_id: body.category_id } : {}),
        ...(body.is_reviewed !== undefined ? { is_reviewed: body.is_reviewed } : {}),
        tag_ids: body.add_tag_ids
          ? [...new Set([...(t.tag_ids ?? []), ...body.add_tag_ids])]
          : t.tag_ids,
        updated_at: new Date().toISOString(),
      };
    }),
  );
  return simulate({ data: { updated: ids.size } });
}
