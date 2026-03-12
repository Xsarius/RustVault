/**
 * Top bar component — user menu, action button, theme toggle.
 */

import { Show } from "solid-js";
import { Plus, LogOut, User, Sun, Moon, Monitor } from "lucide-solid";
import { useNavigate } from "@solidjs/router";
import {
  DropdownMenu,
  DropdownItem,
  DropdownSeparator,
} from "~/components/ui";
import { authStore, themeStore } from "~/stores";
import { useI18n } from "~/i18n";

export function TopBar() {
  const t = useI18n();
  const navigate = useNavigate();
  const user = authStore.user;

  return (
    <header class="h-14 flex items-center justify-between px-4 border-b border-border bg-surface sticky top-0 z-[var(--z-sticky)]">
      {/* Left: page breadcrumb area (filled by pages) */}
      <div class="flex items-center gap-2 lg:hidden">
        <span class="text-base font-semibold text-text">RustVault</span>
      </div>
      <div class="hidden lg:block" />

      {/* Right: actions */}
      <div class="flex items-center gap-2">
        {/* Action menu (+) */}
        <ActionMenu />

        {/* Theme toggle */}
        <ThemeToggle />

        {/* User menu */}
        <Show when={user()}>
          <DropdownMenu
            trigger={
              <button class="flex items-center gap-2 px-2 py-1.5 rounded-[var(--radius-md)] text-sm text-text-secondary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer outline-none">
                <div class="h-7 w-7 rounded-full bg-primary/10 flex items-center justify-center">
                  <User size={14} class="text-primary" />
                </div>
                <span class="hidden sm:inline truncate max-w-[120px]">
                  {user()?.username}
                </span>
              </button>
            }
          >
            <div class="px-3 py-2 text-xs text-text-tertiary truncate">
              {user()?.email}
            </div>
            <DropdownSeparator />
            <DropdownItem onSelect={() => navigate("/settings")}>
              <User size={14} />
              {t("common.nav.settings") ?? "Settings"}
            </DropdownItem>
            <DropdownSeparator />
            <DropdownItem
              onSelect={() => {
                authStore.logout();
                navigate("/login");
              }}
              danger
            >
              <LogOut size={14} />
              Logout
            </DropdownItem>
          </DropdownMenu>
        </Show>
      </div>
    </header>
  );
}

// ── Action Menu (+) ──────────────────────────────────────────

function ActionMenu() {
  const navigate = useNavigate();

  return (
    <DropdownMenu
      trigger={
        <button class="h-8 w-8 flex items-center justify-center rounded-[var(--radius-md)] bg-primary text-white hover:bg-primary-hover transition-colors cursor-pointer">
          <Plus size={18} />
        </button>
      }
    >
      <DropdownItem onSelect={() => navigate("/banks?create=true")}>
        <span class="text-sm">Add Bank</span>
      </DropdownItem>
      <DropdownItem onSelect={() => navigate("/banks?create-account=true")}>
        <span class="text-sm">Add Account</span>
      </DropdownItem>
      <DropdownSeparator />
      <DropdownItem onSelect={() => navigate("/categories?create=true")}>
        <span class="text-sm">Add Category</span>
      </DropdownItem>
      <DropdownItem onSelect={() => navigate("/tags?create=true")}>
        <span class="text-sm">Add Tag</span>
      </DropdownItem>
    </DropdownMenu>
  );
}

// ── Theme Toggle ─────────────────────────────────────────────

function ThemeToggle() {
  const { theme, setTheme } = themeStore;

  const nextTheme = () => {
    const current = theme();
    if (current === "light") return "dark" as const;
    if (current === "dark") return "system" as const;
    return "light" as const;
  };

  const Icon = () => {
    const t = theme();
    if (t === "dark") return <Moon size={16} />;
    if (t === "light") return <Sun size={16} />;
    return <Monitor size={16} />;
  };

  return (
    <button
      onClick={() => setTheme(nextTheme())}
      class="h-8 w-8 flex items-center justify-center rounded-[var(--radius-md)] text-text-secondary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer"
      title={`Theme: ${theme()}`}
    >
      <Icon />
    </button>
  );
}
