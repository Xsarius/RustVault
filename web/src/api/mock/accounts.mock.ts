/**
 * Demo mode — Accounts mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  Account,
  NewAccount,
  UpdateAccount,
  PaginatedResponse,
  ApiResponse,
} from "~/api/types";

function fakeMeta() {
  return { page_size: 50, has_more: false };
}

export async function listAccounts(bankId?: string): Promise<PaginatedResponse<Account>> {
  const data = bankId
    ? demoStore.accounts.filter((a) => a.bank_id === bankId)
    : [...demoStore.accounts];
  return simulate({ data, meta: fakeMeta() });
}

export async function getAccount(id: string): Promise<ApiResponse<Account>> {
  const account = demoStore.accounts.find((a) => a.id === id);
  if (!account) throw new Error("Account not found");
  return simulate({ data: account });
}

export async function createAccount(body: NewAccount): Promise<ApiResponse<Account>> {
  const account: Account = {
    id: `acc-${crypto.randomUUID()}`,
    user_id: "demo-user",
    bank_id: body.bank_id,
    name: body.name,
    currency: body.currency,
    type: body.type,
    balance_cache: "0.00",
    supports_nonstandard_topup: body.supports_nonstandard_topup ?? false,
    is_archived: false,
    sort_order: demoStore.accounts.length,
    metadata: {},
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("accounts", (prev) => [...prev, account]);
  return simulate({ data: account });
}

export async function updateAccount(
  id: string,
  body: UpdateAccount,
): Promise<ApiResponse<Account>> {
  setDemoStore("accounts", (prev) =>
    prev.map((a) =>
      a.id === id ? { ...a, ...body, updated_at: new Date().toISOString() } : a,
    ),
  );
  const updated = demoStore.accounts.find((a) => a.id === id)!;
  return simulate({ data: updated });
}

export async function deleteAccount(id: string): Promise<void> {
  setDemoStore("accounts", (prev) => prev.filter((a) => a.id !== id));
  return simulate(undefined);
}
