/**
 * Categories page — hierarchical tree view with CRUD.
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
  FolderTree,
  Plus,
  ChevronDown,
  ChevronRight,
  Pencil,
  Trash2,
  MoreVertical,
} from "lucide-solid";
import {
  Button,
  Dialog,
  TextField,
  Select,
  ListSkeleton,
  DropdownMenu,
  DropdownItem,
  DropdownSeparator,
  showToast,
} from "~/components/ui";
import { api, type Category } from "~/api";
import { ApiError } from "~/api/client";
import { useI18n } from "~/i18n";

// ── Data ─────────────────────────────────────────────────────

async function fetchCategories(): Promise<Category[]> {
  const res = await api.fetchList<Category>("/api/categories");
  return res.data;
}

/** Build a tree from a flat list of categories. */
function buildTree(categories: Category[]): CategoryNode[] {
  const map = new Map<string, CategoryNode>();
  const roots: CategoryNode[] = [];

  for (const cat of categories) {
    map.set(cat.id, { ...cat, children: [] });
  }

  for (const cat of categories) {
    const node = map.get(cat.id)!;
    if (cat.parent_id && map.has(cat.parent_id)) {
      map.get(cat.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}

interface CategoryNode extends Category {
  children: CategoryNode[];
}

// ── Page ─────────────────────────────────────────────────────

export default function CategoriesPage() {
  const t = useI18n();
  const [searchParams, setSearchParams] = useSearchParams();

  const [categories, { refetch }] = createResource(fetchCategories);
  const tree = () => buildTree(categories() ?? []);

  // ── Create dialog ──────────────────────────────────────────

  const [dialogOpen, setDialogOpen] = createSignal(false);
  const [catName, setCatName] = createSignal("");
  const [catType, setCatType] = createSignal<"income" | "expense">("expense");
  const [catParentId, setCatParentId] = createSignal<string | null>(null);
  const [saving, setSaving] = createSignal(false);

  const openDialog = (parentId?: string) => {
    setCatName("");
    setCatType("expense");
    setCatParentId(parentId ?? null);
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    if (searchParams.create) {
      setSearchParams({ create: undefined });
    }
  };

  const handleCreate = async () => {
    if (!catName().trim()) return;
    setSaving(true);
    try {
      await api.createOne("/api/categories", {
        name: catName().trim(),
        category_type: catType(),
        parent_id: catParentId(),
      });
      showToast({ title: "Category created", variant: "success" });
      closeDialog();
      refetch();
    } catch (err) {
      const msg = err instanceof ApiError ? err.message : "Failed to create category.";
      showToast({ title: msg, variant: "error" });
    } finally {
      setSaving(false);
    }
  };

  // Auto-open from query param
  if (searchParams.create === "true") {
    openDialog();
  }

  // Parent options for select
  const parentOptions = () => {
    const cats = categories() ?? [];
    return [
      { value: "__none__", label: "— None (root) —" },
      ...cats.map((c) => ({ value: c.id, label: c.name })),
    ];
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text">
          {t("common.nav.categories") ?? "Categories"}
        </h1>
        <Button variant="primary" size="sm" onClick={() => openDialog()}>
          <Plus size={16} />
          Add Category
        </Button>
      </div>

      {/* Category tree */}
      <Suspense fallback={<ListSkeleton />}>
        <Show
          when={tree().length > 0}
          fallback={
            <EmptyState onAction={() => openDialog()} />
          }
        >
          <div class="rounded-[var(--radius-lg)] border border-border bg-surface divide-y divide-border">
            <For each={tree()}>
              {(node) => <CategoryTreeItem node={node} depth={0} onRefetch={refetch} />}
            </For>
          </div>
        </Show>
      </Suspense>

      {/* Create dialog */}
      <Dialog
        open={dialogOpen()}
        onOpenChange={(open) => { if (!open) closeDialog(); }}
        title="Add Category"
      >
        <div class="space-y-4 pt-2">
          <TextField
            name="categoryName"
            label="Name"
            value={catName()}
            onInput={(e) => setCatName(e.currentTarget.value)}
            placeholder="e.g. Groceries"
            required
          />
          <Select
            name="categoryType"
            label="Type"
            options={[
              { value: "expense", label: "Expense" },
              { value: "income", label: "Income" },
            ]}
            value={catType()}
            onChange={(v) => setCatType(v as "income" | "expense")}
          />
          <Select
            name="parentCategory"
            label="Parent category"
            options={parentOptions()}
            value={catParentId() ?? "__none__"}
            onChange={(v) => setCatParentId(v === "__none__" ? null : v)}
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
              disabled={!catName().trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

// ── Tree item ────────────────────────────────────────────────

function CategoryTreeItem(props: {
  node: CategoryNode;
  depth: number;
  onRefetch: () => void;
}) {
  const [expanded, setExpanded] = createSignal(true);
  const hasChildren = () => props.node.children.length > 0;
  const indent = () => `${props.depth * 24 + 16}px`;

  const typeColor = () =>
    props.node.category_type === "income"
      ? "text-income"
      : "text-expense";

  return (
    <>
      <div
        class="flex items-center gap-2 py-2.5 pr-4 hover:bg-surface-hover transition-colors"
        style={{ "padding-left": indent() }}
      >
        {/* Expander */}
        <button
          class="w-5 h-5 flex items-center justify-center text-text-tertiary cursor-pointer"
          onClick={() => setExpanded((e) => !e)}
        >
          <Show when={hasChildren()}>
            {expanded() ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          </Show>
        </button>

        {/* Icon */}
        <FolderTree size={16} class={typeColor()} />

        {/* Name */}
        <span class="flex-1 text-sm font-medium text-text truncate">
          {props.node.name}
        </span>

        {/* Type badge */}
        <span
          class={`text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded ${
            props.node.category_type === "income"
              ? "bg-income/10 text-income"
              : "bg-expense/10 text-expense"
          }`}
        >
          {props.node.category_type}
        </span>

        {/* Actions */}
        <DropdownMenu
          trigger={
            <button class="h-6 w-6 flex items-center justify-center rounded text-text-tertiary hover:text-text hover:bg-surface-hover cursor-pointer">
              <MoreVertical size={14} />
            </button>
          }
        >
          <DropdownItem onSelect={() => {}}>
            <Pencil size={14} />
            Edit
          </DropdownItem>
          <DropdownSeparator />
          <DropdownItem onSelect={() => {}} danger>
            <Trash2 size={14} />
            Delete
          </DropdownItem>
        </DropdownMenu>
      </div>

      {/* Children */}
      <Show when={expanded() && hasChildren()}>
        <For each={props.node.children}>
          {(child) => (
            <CategoryTreeItem
              node={child}
              depth={props.depth + 1}
              onRefetch={props.onRefetch}
            />
          )}
        </For>
      </Show>
    </>
  );
}

// ── Empty state ──────────────────────────────────────────────

function EmptyState(props: { onAction: () => void }) {
  return (
    <div class="flex flex-col items-center justify-center py-16 text-center">
      <FolderTree size={48} class="text-text-tertiary mb-4" />
      <h2 class="text-lg font-semibold text-text">No categories yet</h2>
      <p class="text-sm text-text-secondary mt-1 max-w-xs">
        Create income and expense categories to organize your transactions.
      </p>
      <Button variant="primary" size="sm" class="mt-4" onClick={props.onAction}>
        <Plus size={16} />
        Add Category
      </Button>
    </div>
  );
}
