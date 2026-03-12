/**
 * Sidebar navigation component.
 *
 * Responsive: expanded (240px) on xl, collapsed (56px icon-only) on lg,
 * hidden on mobile (replaced by bottom tab bar).
 */

import { type Component, For } from "solid-js";
import { A, useLocation } from "@solidjs/router";
import {
  LayoutDashboard,
  ArrowLeftRight,
  PiggyBank,
  BarChart3,
  Landmark,
  FolderTree,
  Tags,
  Settings,
  PanelLeftClose,
  PanelLeft,
} from "lucide-solid";
import logoSvg from "./logo.svg";
import { Tooltip } from "~/components/ui";
import { useI18n } from "~/i18n";

interface NavItem {
  path: string;
  labelKey: string;
  icon: Component<{ size?: number }>;
}

const NAV_ITEMS: NavItem[] = [
  { path: "/", labelKey: "common.nav.dashboard", icon: LayoutDashboard },
  { path: "/transactions", labelKey: "common.nav.transactions", icon: ArrowLeftRight },
  { path: "/budget", labelKey: "common.nav.budget", icon: PiggyBank },
  { path: "/reports", labelKey: "common.nav.reports", icon: BarChart3 },
];

const MANAGE_ITEMS: NavItem[] = [
  { path: "/banks", labelKey: "common.nav.banks", icon: Landmark },
  { path: "/categories", labelKey: "common.nav.categories", icon: FolderTree },
  { path: "/tags", labelKey: "common.nav.tags", icon: Tags },
];

const BOTTOM_ITEMS: NavItem[] = [
  { path: "/settings", labelKey: "common.nav.settings", icon: Settings },
];

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
}

export function Sidebar(props: SidebarProps) {
  const t = useI18n();
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === "/") return location.pathname === "/";
    return location.pathname.startsWith(path);
  };

  return (
    <aside
      class="hidden lg:flex flex-col border-r border-border bg-surface h-screen sticky top-0 transition-[width] duration-200"
      style={{ width: props.collapsed ? "56px" : "240px" }}
    >
      {/* Logo */}
      <div class="flex items-center h-14 px-4 border-b border-border">
        {!props.collapsed && (
          <div class="flex items-center gap-2">
            <img src={logoSvg} alt="RustVault" class="size-7" />
            <span class="text-base font-semibold text-text truncate">RustVault</span>
          </div>
        )}
        {props.collapsed && (
          <img src={logoSvg} alt="RustVault" class="size-7 mx-auto" />
        )}
      </div>

      {/* Main navigation */}
      <nav class="flex-1 flex flex-col gap-0.5 px-2 pt-3 overflow-y-auto">
        <For each={NAV_ITEMS}>
          {(item) => (
            <NavLink
              item={item}
              active={isActive(item.path)}
              collapsed={props.collapsed}
              label={(t(item.labelKey as any) as string | undefined) ?? ""}
            />
          )}
        </For>

        {/* Spacing between groups */}
        <div class="h-4" />

        <For each={MANAGE_ITEMS}>
          {(item) => (
            <NavLink
              item={item}
              active={isActive(item.path)}
              collapsed={props.collapsed}
              label={(t(item.labelKey as any) as string | undefined) ?? ""}
            />
          )}
        </For>
      </nav>

      {/* Bottom section */}
      <div class="flex flex-col gap-0.5 px-2 pb-2">
        <For each={BOTTOM_ITEMS}>
          {(item) => (
            <NavLink
              item={item}
              active={isActive(item.path)}
              collapsed={props.collapsed}
              label={(t(item.labelKey as any) as string | undefined) ?? ""}
            />
          )}
        </For>

        {/* Collapse toggle */}
        <button
          onClick={props.onToggle}
          class="flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] text-text-tertiary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer"
        >
          {props.collapsed ? <PanelLeft size={18} /> : <PanelLeftClose size={18} />}
          {!props.collapsed && <span class="text-sm">Collapse</span>}
        </button>
      </div>
    </aside>
  );
}

// ── NavLink sub-component ─────────────────────────────────────

interface NavLinkProps {
  item: NavItem;
  active: boolean;
  collapsed: boolean;
  label: string;
}

function NavLink(props: NavLinkProps) {
  const Icon = props.item.icon;

  const link = (
    <A
      href={props.item.path}
      class={`flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] text-sm font-medium transition-colors ${
        props.active
          ? "bg-primary/10 text-primary border-l-2 border-primary"
          : "text-text-secondary hover:text-text hover:bg-surface-hover"
      } ${props.collapsed ? "justify-center" : ""}`}
    >
      <Icon size={18} />
      {!props.collapsed && <span class="truncate">{props.label}</span>}
    </A>
  );

  if (props.collapsed) {
    return <Tooltip content={props.label}>{link}</Tooltip>;
  }

  return link;
}
