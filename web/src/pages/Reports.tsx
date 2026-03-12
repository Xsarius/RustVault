/**
 * Reports page — placeholder for Phase 5.
 */

import { BarChart3 } from "lucide-solid";
import { useI18n } from "~/i18n";

export default function ReportsPage() {
  const t = useI18n();

  return (
    <div class="flex flex-col items-center justify-center py-24 text-center">
      <BarChart3 size={48} class="text-text-tertiary mb-4" />
      <h1 class="text-2xl font-bold text-text">
        {t("common.nav.reports") ?? "Reports"}
      </h1>
      <p class="text-sm text-text-secondary mt-2 max-w-xs">
        Reports and analytics will be available in Phase 5.
      </p>
    </div>
  );
}
