/**
 * Demo mode — Categories mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  Category,
  NewCategory,
  UpdateCategory,
  PaginatedResponse,
  ApiResponse,
} from "~/api/types";

function fakeMeta() {
  return { page_size: 100, has_more: false };
}

export async function listCategories(): Promise<PaginatedResponse<Category>> {
  return simulate({ data: [...demoStore.categories], meta: fakeMeta() });
}

export async function getCategory(id: string): Promise<ApiResponse<Category>> {
  const cat = demoStore.categories.find((c) => c.id === id);
  if (!cat) throw new Error("Category not found");
  return simulate({ data: cat });
}

export async function createCategory(body: NewCategory): Promise<ApiResponse<Category>> {
  const cat: Category = {
    id: `cat-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    parent_id: body.parent_id ?? null,
    icon: body.icon ?? null,
    color: body.color ?? null,
    category_type: body.category_type,
    sort_order: demoStore.categories.length,
    metadata: {},
    created_at: new Date().toISOString(),
  };
  setDemoStore("categories", (prev) => [...prev, cat]);
  return simulate({ data: cat });
}

export async function updateCategory(
  id: string,
  body: UpdateCategory,
): Promise<ApiResponse<Category>> {
  setDemoStore("categories", (prev) =>
    prev.map((c) => (c.id === id ? { ...c, ...body } : c)),
  );
  const updated = demoStore.categories.find((c) => c.id === id)!;
  return simulate({ data: updated });
}

export async function deleteCategory(id: string): Promise<void> {
  setDemoStore("categories", (prev) => prev.filter((c) => c.id !== id));
  return simulate(undefined);
}

export async function bulkCreateCategories(
  categories: NewCategory[],
): Promise<PaginatedResponse<Category>> {
  const created: Category[] = categories.map((body) => ({
    id: `cat-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    parent_id: body.parent_id ?? null,
    icon: body.icon ?? null,
    color: body.color ?? null,
    category_type: body.category_type,
    sort_order: demoStore.categories.length,
    metadata: {},
    created_at: new Date().toISOString(),
  }));
  setDemoStore("categories", (prev) => [...prev, ...created]);
  return simulate({ data: created, meta: fakeMeta() });
}
