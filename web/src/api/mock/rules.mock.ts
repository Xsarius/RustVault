/**
 * Demo mode — Rules mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  AutoRule,
  NewAutoRule,
  UpdateAutoRule,
  TestRuleResponse,
  PaginatedResponse,
  ApiResponse,
} from "~/api/types";

function fakeMeta() {
  return { page_size: 50, has_more: false };
}

export async function listRules(): Promise<PaginatedResponse<AutoRule>> {
  return simulate({ data: [...demoStore.rules], meta: fakeMeta() });
}

export async function getRule(id: string): Promise<ApiResponse<AutoRule>> {
  const rule = demoStore.rules.find((r) => r.id === id);
  if (!rule) throw new Error("Rule not found");
  return simulate({ data: rule });
}

export async function createRule(body: NewAutoRule): Promise<ApiResponse<AutoRule>> {
  const rule: AutoRule = {
    id: `rule-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    priority: body.priority ?? 50,
    is_enabled: true,
    conditions: body.conditions,
    actions: body.actions,
    metadata: {},
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  setDemoStore("rules", (prev) => [...prev, rule]);
  return simulate({ data: rule });
}

export async function updateRule(
  id: string,
  body: UpdateAutoRule,
): Promise<ApiResponse<AutoRule>> {
  setDemoStore("rules", (prev) =>
    prev.map((r) =>
      r.id === id ? { ...r, ...body, updated_at: new Date().toISOString() } : r,
    ),
  );
  const updated = demoStore.rules.find((r) => r.id === id)!;
  return simulate({ data: updated });
}

export async function deleteRule(id: string): Promise<void> {
  setDemoStore("rules", (prev) => prev.filter((r) => r.id !== id));
  return simulate(undefined);
}

export async function testRule(
  conditions: unknown,
  description: string,
  _payee?: string,
): Promise<ApiResponse<TestRuleResponse>> {
  // Simple heuristic: if conditions is an array, check first condition's value against description
  let matched = false;
  if (Array.isArray(conditions) && conditions.length > 0) {
    const val = String(conditions[0].value ?? "").toLowerCase();
    matched = description.toLowerCase().includes(val);
  }
  return simulate({ data: { matched } });
}
