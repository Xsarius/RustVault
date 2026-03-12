/**
 * Budget page — placeholder for Phase 4.
 */

import { PiggyBank } from "lucide-solid";
import { useI18n } from "~/i18n";

export default function BudgetPage() {
  const t = useI18n();

  return (
    <div class="flex flex-col items-center justify-center py-24 text-center">
      <PiggyBank size={48} class="text-text-tertiary mb-4" />
      <h1 class="text-2xl font-bold text-text">
        {t("common.nav.budget") ?? "Budget"}
      </h1>
      <p class="text-sm text-text-secondary mt-2 max-w-xs">
        Budget management will be available in Phase 4.
      </p>
    </div>
  );
}
