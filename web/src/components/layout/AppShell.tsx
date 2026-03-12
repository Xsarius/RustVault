/**
 * AppShell — root layout component.
 *
 * Composes Sidebar (desktop) + TopBar + BottomTabBar (mobile)
 * around the main content area. Handles responsive breakpoints
 * and sidebar collapsed state.
 */

import {
  type ParentComponent,
  createSignal,
  createEffect,
  onCleanup,
} from "solid-js";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { BottomTabBar } from "./BottomTabBar";

const XL_BREAKPOINT = 1280;

export const AppShell: ParentComponent = (props) => {
  // Auto-collapse sidebar on lg; expand on xl+
  const [collapsed, setCollapsed] = createSignal(
    typeof window !== "undefined" ? window.innerWidth < XL_BREAKPOINT : false,
  );

  createEffect(() => {
    const mq = window.matchMedia(`(min-width: ${XL_BREAKPOINT}px)`);
    const handler = (e: MediaQueryListEvent) => setCollapsed(!e.matches);
    mq.addEventListener("change", handler);
    onCleanup(() => mq.removeEventListener("change", handler));
  });

  return (
    <div class="flex min-h-screen bg-bg text-text">
      {/* Sidebar — desktop only (lg+) */}
      <Sidebar
        collapsed={collapsed()}
        onToggle={() => setCollapsed((c) => !c)}
      />

      {/* Main column */}
      <div class="flex flex-col flex-1 min-w-0">
        <TopBar />

        <main class="flex-1 p-4 lg:p-6 pb-20 lg:pb-6">
          {props.children}
        </main>
      </div>

      {/* Bottom tab bar — mobile only */}
      <BottomTabBar />
    </div>
  );
};
