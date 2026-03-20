/**
 * Demo mode — Tags mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type {
  Tag,
  NewTag,
  UpdateTag,
  PaginatedResponse,
  ApiResponse,
} from "~/api/types";

function fakeMeta() {
  return { page_size: 100, has_more: false };
}

export async function listTags(): Promise<PaginatedResponse<Tag>> {
  return simulate({ data: [...demoStore.tags], meta: fakeMeta() });
}

export async function getTag(id: string): Promise<ApiResponse<Tag>> {
  const tag = demoStore.tags.find((t) => t.id === id);
  if (!tag) throw new Error("Tag not found");
  return simulate({ data: tag });
}

export async function createTag(body: NewTag): Promise<ApiResponse<Tag>> {
  const tag: Tag = {
    id: `tag-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    color: body.color ?? null,
    created_at: new Date().toISOString(),
  };
  setDemoStore("tags", (prev) => [...prev, tag]);
  return simulate({ data: tag });
}

export async function updateTag(id: string, body: UpdateTag): Promise<ApiResponse<Tag>> {
  setDemoStore("tags", (prev) =>
    prev.map((t) => (t.id === id ? { ...t, ...body } : t)),
  );
  const updated = demoStore.tags.find((t) => t.id === id)!;
  return simulate({ data: updated });
}

export async function deleteTag(id: string): Promise<void> {
  setDemoStore("tags", (prev) => prev.filter((t) => t.id !== id));
  return simulate(undefined);
}

export async function bulkCreateTags(
  tags: NewTag[],
): Promise<PaginatedResponse<Tag>> {
  const created: Tag[] = tags.map((body) => ({
    id: `tag-${crypto.randomUUID()}`,
    user_id: "demo-user",
    name: body.name,
    color: body.color ?? null,
    created_at: new Date().toISOString(),
  }));
  setDemoStore("tags", (prev) => [...prev, ...created]);
  return simulate({ data: created, meta: fakeMeta() });
}
