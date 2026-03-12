/**
 * "More" page — mobile-only (/more route).
 *
 * Shown when tapping "More" in the bottom tab bar.
 * Links to manage sections + settings not shown in bottom tabs.
 */

import { A } from "@solidjs/router";
import {
  Landmark,
  FolderTree,
  Tags,
  Settings,
  ChevronRight,
} from "lucide-solid";
import { useI18n } from "~/i18n";

const ITEMS = [
  { path: "/banks", labelKey: "common.nav.banks", icon: Landmark },
  { path: "/categories", labelKey: "common.nav.categories", icon: FolderTree },
  { path: "/tags", labelKey: "common.nav.tags", icon: Tags },
  { path: "/settings", labelKey: "common.nav.settings", icon: Settings },
] as const;

export default function MorePage() {
  const t = useI18n();

  return (
    <div class="space-y-4">
      <h1 class="text-2xl font-bold text-text">More</h1>
      <div class="rounded-[var(--radius-lg)] border border-border bg-surface divide-y divide-border">
        {ITEMS.map((item) => (
          <A
            href={item.path}
            class="flex items-center gap-3 px-4 py-3 hover:bg-surface-hover transition-colors"
          >
            <item.icon size={18} class="text-text-secondary shrink-0" />
            <span class="flex-1 text-sm font-medium text-text">
              {t(item.labelKey) ?? item.labelKey}
            </span>
            <ChevronRight size={16} class="text-text-tertiary" />
          </A>
        ))}
      </div>
    </div>
  );
}
