/**
 * Demo mode — Banks mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type { Bank, NewBank, UpdateBank, PaginatedResponse, ApiResponse } from "~/api/types";

function fakeMeta() {
  return { page_size: 50, has_more: false };
}

export async function listBanks(): Promise<PaginatedResponse<Bank>> {
  return simulate({ data: [...demoStore.banks], meta: fakeMeta() });
}

export async function getBank(id: string): Promise<ApiResponse<Bank>> {
  const bank = demoStore.banks.find((b) => b.id === id);
  if (!bank) throw new Error("Not found");
  return simulate({ data: bank });
}

export async function createBank(body: NewBank): Promise<ApiResponse<Bank>> {
  const bank: Bank = {
    id: `bank-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    is_archived: false,
    sort_order: demoStore.banks.length,
    metadata: {},
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("banks", (prev) => [...prev, bank]);
  return simulate({ data: bank });
}

export async function updateBank(id: string, body: UpdateBank): Promise<ApiResponse<Bank>> {
  setDemoStore("banks", (prev) =>
    prev.map((b) => (b.id === id ? { ...b, ...body, updated_at: new Date().toISOString() } : b)),
  );
  const updated = demoStore.banks.find((b) => b.id === id)!;
  return simulate({ data: updated });
}

export async function deleteBank(id: string): Promise<void> {
  setDemoStore("banks", (prev) => prev.filter((b) => b.id !== id));
  return simulate(undefined);
}
