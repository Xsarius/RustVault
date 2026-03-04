import type { Component } from "solid-js";
import { useI18n, useLocale } from "~/i18n";

const App: Component = () => {
  const t = useI18n();
  const { locale } = useLocale();

  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <div class="text-center">
        <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
          {t("common.app.name")}
        </h1>
        <p class="text-lg text-gray-600 dark:text-gray-400">
          {t("common.app.description")}
        </p>
        <p class="mt-6 text-sm text-gray-500 dark:text-gray-500">
          Hello {t("common.app.name")} — Phase 0 scaffolding complete.
        </p>
        <p class="mt-2 text-xs text-gray-400 dark:text-gray-600">
          Locale: {locale()}
        </p>
      </div>
    </div>
  );
};

export default App;
