import type { Component } from "solid-js";

const App: Component = () => {
  return (
    <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <div class="text-center">
        <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
          RustVault
        </h1>
        <p class="text-lg text-gray-600 dark:text-gray-400">
          Self-hosted personal finance platform
        </p>
        <p class="mt-6 text-sm text-gray-500 dark:text-gray-500">
          Hello RustVault — Phase 0 scaffolding complete.
        </p>
      </div>
    </div>
  );
};

export default App;
