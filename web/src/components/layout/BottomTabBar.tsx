/**
 * Bottom tab bar — shown on mobile (below sm breakpoint).
 */

import { A, useLocation } from "@solidjs/router";
import {
  LayoutDashboard,
  ArrowLeftRight,
  Plus,
  PiggyBank,
  Menu,
} from "lucide-solid";

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

export function BottomTabBar() {
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return location.pathname.startsWith(path);
  };

  return (
    <nav class="lg:hidden fixed bottom-0 inset-x-0 z-[var(--z-sticky)] flex items-center justify-around h-14 border-t border-border bg-surface safe-area-pb">
      {TABS.map((tab) =>
        tab.accent ? (
          <button
            class="flex items-center justify-center h-10 w-10 -mt-3 rounded-full bg-primary text-white shadow-sm cursor-pointer"
            onClick={() => {
              // TODO: Open action bottom sheet on mobile
            }}
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
  );
}
