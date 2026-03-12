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
  Tags as TagsIcon,
  Plus,
  X,
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

// ── Page ─────────────────────────────────────────────────────

export default function TagsPage() {
  const t = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();

  const [tags, { refetch }] = createResource(fetchTags);

  // ── Create dialog ──────────────────────────────────────────

  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [tagName, setTagName] = createSignal("");
  const [saving, setSaving] = createSignal(false);

  const openDialog = () => {
    setTagName("");
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    if (searchParams.create) {
      setSearchParams({ create: undefined });
    }
  };

  const handleCreate = async () => {
    if (!tagName().trim()) return;
    setSaving(true);
    try {
      await api.createOne("/api/tags", { name: tagName().trim() });
      showToast({ title: "Tag created", variant: "success" });
      closeDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to create tag.";
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
        <Button variant="primary" size="sm" onClick={openDialog}>
          <Plus size={16} />
          Add Tag
        </Button>
      </div>

      {/* Tag list */}
      <Suspense fallback={<ListSkeleton />}>
        <Show
          when={tags() && tags()!.length > 0}
          fallback={<EmptyState onAction={openDialog} />}
        >
          <div class="flex flex-wrap gap-2">
            <For each={tags()}>
              {(tag) => <TagChip tag={tag} onDelete={handleDelete} />}
            </For>
          </div>
        </Show>
      </Suspense>

      {/* Create dialog */}
      <Dialog
        open={dialogOpen()}
        onOpenChange={(open) => { if (!open) closeDialog(); }}
        title="Add Tag"
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
          <div class="flex justify-end gap-2">
            <Button variant="secondary" size="sm" onClick={closeDialog}>
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleCreate}
              loading={saving()}
              disabled={!tagName().trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── Tag chip ─────────────────────────────────────────────────

function TagChip(props: { tag: Tag; onDelete: (tag: Tag) => void }) {
  return (
    <div class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full border border-border bg-surface text-sm font-medium text-text group">
      <TagsIcon size={14} class="text-text-tertiary" />
      <span>{props.tag.name}</span>
      <button
        onClick={() => props.onDelete(props.tag)}
        class="ml-0.5 h-4 w-4 flex items-center justify-center rounded-full opacity-0 group-hover:opacity-100 text-text-tertiary hover:text-danger transition-all cursor-pointer"
        title="Delete tag"
      >
        <X size={12} />
      </button>
    </div>
  );
}

// ── Empty state ──────────────────────────────────────────────

function EmptyState(props: { onAction: () => void }) {
  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <TagsIcon size={48} class="text-text-tertiary mb-4" />
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
