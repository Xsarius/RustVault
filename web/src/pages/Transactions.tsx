/**
 * Transactions page — placeholder for Phase 3.
 */

import { ArrowLeftRight } from "lucide-solid";
import { useI18n } from "~/i18n";

export default function TransactionsPage() {
  const t = useI18n();

  return (
    <div class="flex flex-col items-center justify-center py-24 text-center">
      <ArrowLeftRight size={48} class="text-text-tertiary mb-4" />
      <h1 class="text-2xl font-bold text-text">
        {t("common.nav.transactions") ?? "Transactions"}
      </h1>
      <p class="text-sm text-text-secondary mt-2 max-w-xs">
        Transaction management will be available in Phase 3.
      </p>
    </div>
  );
}
