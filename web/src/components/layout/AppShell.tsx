/**
 * AppShell — root layout component.
 *
 * Composes Sidebar (desktop) + TopBar + BottomTabBar (mobile)
 * around the main content area. Handles responsive breakpoints
 * and sidebar collapsed state.
 *
 * On Capacitor native builds the `viewport-fit=cover` meta tag is set
 * in index.html so the app extends behind the device status bar and
 * home indicator. This component adds bottom padding via the
 * `safe-area-inset-bottom` env() variable.
 */

import {
  type ParentComponent,
  createSignal,
  createEffect,
  onCleanup,
  onMount,
} from "solid-js";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { BottomTabBar } from "./BottomTabBar";
import { OfflineBanner } from "~/components/mobile";
import { isMobile } from "~/mobile";
import { useOfflineSync } from "~/mobile";

const XL_BREAKPOINT = 1280;

export const AppShell: ParentComponent = (props) => {
  // Auto-collapse sidebar on lg; expand on xl+
  const [collapsed, setCollapsed] = createSignal(
    typeof window !== "undefined" ? window.innerWidth < XL_BREAKPOINT : false,
  );

  const offline = useOfflineSync();

  createEffect(() => {
    const mq = window.matchMedia(`(min-width: ${XL_BREAKPOINT}px)`);
    const handler = (e: MediaQueryListEvent) => setCollapsed(!e.matches);
    mq.addEventListener("change", handler);
    onCleanup(() => mq.removeEventListener("change", handler));
  });

  // On Capacitor native, prevent the default browser bounce scroll and
  // provide a data attribute that CSS can use to adjust safe-area padding.
  onMount(() => {
    if (isMobile()) {
      document.documentElement.setAttribute("data-capacitor", "true");
    }
  });

  return (
    <div class="flex min-h-screen bg-bg text-text">
      <OfflineBanner
        isOnline={offline.isOnline()}
        isSyncing={offline.isSyncing()}
        pendingCount={offline.pendingCount()}
        onSync={offline.sync}
      />

      {/* Sidebar — desktop only (lg+) */}
      <Sidebar
        collapsed={collapsed()}
        onToggle={() => setCollapsed((c) => !c)}
      />

      {/* Main column */}
      <div class="flex flex-col flex-1 min-w-0">
        <TopBar />

        {/*
          pb-20 on mobile leaves room for the bottom tab bar (h-14 = 56px)
          plus the device's safe-area-inset-bottom. On desktop pb-6 suffices.
        */}
        <main class="flex-1 p-4 lg:p-6 pb-[calc(5rem+env(safe-area-inset-bottom,0px))] lg:pb-6">
          {props.children}
        </main>
      </div>

      {/* Bottom tab bar — mobile only */}
      <BottomTabBar />
    </div>
  );
};
