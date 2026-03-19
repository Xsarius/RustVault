/**
 * Bottom tab bar — shown on mobile (below lg breakpoint).
 */

import { createSignal, Show, For } from "solid-js";
import { A, useLocation, useNavigate } from "@solidjs/router";
import {
  LayoutDashboard,
  ArrowLeftRight,
  Plus,
  PiggyBank,
  Menu,
  X,
  Upload,
  FolderTree,
  Tags,
  Landmark,
} from "lucide-solid";
import { useI18n } from "~/i18n";

interface TabItem {
  path: string;
  icon: typeof LayoutDashboard;
  label: string;
  accent?: boolean;
}

const TABS: TabItem[] = [
  { path: "/", icon: LayoutDashboard, label: "Home" },
  { path: "/transactions", icon: ArrowLeftRight, label: "Transactions" },
  { path: "/__action__", icon: Plus, label: "Add", accent: true },
  { path: "/budget", icon: PiggyBank, label: "Budget" },
  { path: "/more", icon: Menu, label: "More" },
];

interface ActionItem {
  label: string;
  labelKey?: string;
  icon: typeof Plus;
  path: string;
  query?: string;
}

const ACTION_ITEMS: ActionItem[] = [
  { label: "Import File", labelKey: "common.mobile.importFile", icon: Upload, path: "/transactions", query: "?import=true" },
  { label: "Add Bank", labelKey: "common.mobile.addBank", icon: Landmark, path: "/banks", query: "?create=true" },
  { label: "Add Category", labelKey: "common.mobile.addCategory", icon: FolderTree, path: "/categories", query: "?create=true" },
  { label: "Add Tag", labelKey: "common.mobile.addTag", icon: Tags, path: "/tags", query: "?create=true" },
];

export function BottomTabBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const t = useI18n();
  const [actionSheetOpen, setActionSheetOpen] = createSignal(false);

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return location.pathname.startsWith(path);
  };

  return (
    <>
      <nav class="lg:hidden fixed bottom-0 inset-x-0 z-[var(--z-sticky)] flex items-center justify-around h-14 border-t border-border bg-surface safe-area-pb">
        {TABS.map((tab) =>
          tab.accent ? (
            <button
              class="flex items-center justify-center h-10 w-10 -mt-3 rounded-full bg-primary text-white shadow-sm cursor-pointer"
              onClick={() => setActionSheetOpen(true)}
            >
              <Plus size={20} />
            </button>
          ) : (
            <A
              href={tab.path}
              class={`flex flex-col items-center gap-0.5 py-1 px-2 text-[10px] font-medium transition-colors ${
                isActive(tab.path) ? "text-primary" : "text-text-tertiary"
              }`}
            >
              <tab.icon size={20} />
              {tab.label}
            </A>
          ),
        )}
      </nav>

      {/* Action bottom sheet */}
      <Show when={actionSheetOpen()}>
        {/* Backdrop */}
        <div
          class="lg:hidden fixed inset-0 z-[var(--z-modal)] bg-black/40"
          onClick={() => setActionSheetOpen(false)}
        />

        {/* Sheet */}
        <div class="lg:hidden fixed bottom-0 inset-x-0 z-[calc(var(--z-modal)+1)] bg-surface rounded-t-2xl border-t border-border pb-safe">
          {/* Handle */}
          <div class="flex justify-center pt-3 pb-1">
            <div class="h-1 w-10 rounded-full bg-border" />
          </div>

          {/* Header */}
          <div class="flex items-center justify-between px-4 py-2">
            <span class="text-sm font-semibold text-text">
              {t("common.mobile.quickActions") ?? "Quick Actions"}
            </span>
            <button
              class="text-text-tertiary hover:text-text cursor-pointer"
              onClick={() => setActionSheetOpen(false)}
            >
              <X size={18} />
            </button>
          </div>

          {/* Action list */}
          <div class="px-2 pb-6 space-y-1">
            <For each={ACTION_ITEMS}>
              {(item) => (
                <button
                  class="w-full flex items-center gap-3 px-4 py-3 rounded-[var(--radius-lg)] text-left text-sm text-text hover:bg-surface-hover transition-colors cursor-pointer"
                  onClick={() => {
                    setActionSheetOpen(false);
                    navigate(`${item.path}${item.query ?? ""}`);
                  }}
                >
                  <item.icon size={20} class="text-primary" />
                  {item.labelKey ? (t(item.labelKey as any) ?? item.label) : item.label}
                </button>
              )}
            </For>
          </div>
        </div>
      </Show>
    </>
  );
}
