/**
 * Tags page — flat tag list with inline CRUD.
 */

import {
  createSignal,
  createResource,
  For,
  Show,
  Suspense,
} from "solid-js";
import { useSearchParams } from "@solidjs/router";
import {
  Plus,
  X,
  Pencil,
} from "lucide-solid";
import {
  Button,
  Dialog,
  TextField,
  ListSkeleton,
  showToast,
} from "~/components/ui";
import { api, type Tag } from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";

// ── Data ─────────────────────────────────────────────────────

async function fetchTags(): Promise<Tag[]> {
  const res = await api.fetchList<Tag>("/api/tags");
  return res.data;
}

// Preset colors shown in the color picker row
const PRESET_COLORS = [
  "#2563eb", "#7c3aed", "#16a34a", "#0891b2",
  "#d97706", "#db2777", "#dc2626", "#64748b",
  "#ea580c", "#65a30d", "#0d9488", "#9333ea",
];

// ── Page ─────────────────────────────────────────────────────

export default function TagsPage() {
  const t = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();

  const [tags, { refetch }] = createResource(fetchTags);

  // ── Create / Edit dialog state ─────────────────────────────

  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [editingTag, setEditingTag] = createSignal<Tag | null>(null);
  const [tagName, setTagName] = createSignal("");
  const [tagColor, setTagColor] = createSignal("#2563eb");
  const [saving, setSaving] = createSignal(false);

  const openDialog = (tag?: Tag) => {
    setEditingTag(tag ?? null);
    setTagName(tag?.name ?? "");
    setTagColor(tag?.color ?? "#2563eb");
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setEditingTag(null);
    setDialogOpen(false);
    if (searchParams.create) {
      setSearchParams({ create: undefined });
    }
  };

  const handleSave = async () => {
    if (!tagName().trim()) return;
    setSaving(true);
    try {
      const existing = editingTag();
      if (existing) {
        await api.updateOne(`/api/tags/${existing.id}`, { name: tagName().trim(), color: tagColor() });
        showToast({ title: "Tag updated", variant: "success" });
      } else {
        await api.createOne("/api/tags", { name: tagName().trim(), color: tagColor() });
        showToast({ title: "Tag created", variant: "success" });
      }
      closeDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to save tag.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (tag: Tag) => {
    try {
      await api.del(`/api/tags/${tag.id}`);
      showToast({ title: `Tag "${tag.name}" deleted`, variant: "success" });
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to delete tag.";
      showToast({ title: msg, variant: "error" });
    }
  };

  // Auto-open from query param
  if (searchParams.create === "true") {
    openDialog();
  }

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("common.nav.tags") ?? "Tags"}
        </h1>
        <Button variant="primary" size="sm" onClick={() => openDialog()}>
          <Plus size={16} />
          Add Tag
        </Button>
      </div>

      {/* Tag grid */}
      <Suspense fallback={<ListSkeleton />}>
        <Show
          when={tags() && tags()!.length > 0}
          fallback={<EmptyState onAction={() => openDialog()} />}
        >
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            <For each={tags()}>
              {(tag) => (
                <TagCard
                  tag={tag}
                  onEdit={() => openDialog(tag)}
                  onDelete={() => handleDelete(tag)}
                />
              )}
            </For>
          </div>
        </Show>
      </Suspense>

      {/* Create / Edit dialog */}
      <Dialog
        open={dialogOpen()}
        onOpenChange={(open) => { if (!open) closeDialog(); }}
        title={editingTag() ? "Edit Tag" : "Add Tag"}
      >
        <div class="space-y-4 pt-2">
          <TextField
            name="tagName"
            label="Tag name"
            value={tagName()}
            onInput={(e) => setTagName(e.currentTarget.value)}
            placeholder="e.g. Subscription"
            required
          />

          {/* Color picker */}
          <div class="space-y-2">
            <label class="text-sm font-medium text-text">Color</label>
            <div class="flex flex-wrap gap-2">
              <For each={PRESET_COLORS}>
                {(color) => (
                  <button
                    type="button"
                    class="h-7 w-7 rounded-full border-2 transition-transform hover:scale-110 cursor-pointer"
                    style={{ "background-color": color, "border-color": tagColor() === color ? "white" : "transparent" }}
                    classList={{ "scale-110 ring-2 ring-offset-1 ring-accent": tagColor() === color }}
                    onClick={() => setTagColor(color)}
                    title={color}
                  />
                )}
              </For>
              {/* Custom color via native input */}
              <label class="h-7 w-7 rounded-full border-2 border-border cursor-pointer overflow-hidden hover:scale-110 transition-transform" title="Custom color">
                <input
                  type="color"
                  value={tagColor()}
                  onInput={(e) => setTagColor(e.currentTarget.value)}
                  class="h-8 w-8 -mt-0.5 -ml-0.5 cursor-pointer opacity-0 absolute"
                />
                <span
                  class="block h-full w-full rounded-full"
                  style={{ "background-color": tagColor() }}
                />
              </label>
            </div>
            <div class="flex items-center gap-2 mt-1">
              <span
                class="h-4 w-4 rounded-full flex-shrink-0"
                style={{ "background-color": tagColor() }}
              />
              <span class="text-xs text-text-secondary font-mono">{tagColor()}</span>
            </div>
          </div>

          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={closeDialog}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSave}
              loading={saving()}
              disabled={!tagName().trim()}
            >
              {editingTag() ? "Save" : "Create"}
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── Tag card ─────────────────────────────────────────────────

function TagCard(props: { tag: Tag; onEdit: () => void; onDelete: () => void }) {
  const color = () => props.tag.color ?? "#64748b";
  return (
    <div
      class="group flex items-center gap-3 rounded-[var(--radius-md)] border border-border bg-surface px-4 py-3 hover:bg-surface-hover transition-colors"
      style={{ "border-left": `4px solid ${color()}` }}
    >
      {/* Color dot */}
      <span
        class="h-3 w-3 rounded-full flex-shrink-0"
        style={{ "background-color": color() }}
      />

      {/* Name */}
      <span class="flex-1 text-sm font-medium text-text truncate">{props.tag.name}</span>

      {/* Actions — visible on hover */}
      <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={props.onEdit}
          class="h-6 w-6 flex items-center justify-center rounded text-text-tertiary hover:text-text hover:bg-surface-hover cursor-pointer transition-colors"
          title="Edit tag"
        >
          <Pencil size={13} />
        </button>
        <button
          onClick={props.onDelete}
          class="h-6 w-6 flex items-center justify-center rounded text-text-tertiary hover:text-danger cursor-pointer transition-colors"
          title="Delete tag"
        >
          <X size={13} />
        </button>
      </div>
    </div>
  );
}

// ── Empty state ──────────────────────────────────────────────

function EmptyState(props: { onAction: () => void }) {
  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <h2 class="text-lg font-semibold text-text">No tags yet</h2>
      <p class="text-sm text-text-secondary mt-1 max-w-xs">
        Tags let you add flexible labels to transactions beyond categories.
      </p>
      <Button variant="primary" size="sm" class="mt-4" onClick={props.onAction}>
        <Plus size={16} />
        Add Tag
      </Button>
    </div>
  );
}
