/**
 * Auth layout — centered card with no sidebar.
 * Used for login, register, and OIDC callback pages.
 */

import type { ParentComponent } from "solid-js";

export const AuthLayout: ParentComponent = (props) => {
  return (
    <div class="min-h-screen flex items-center justify-center bg-bg px-4">
      <div class="w-full max-w-sm">
        {/* Logo */}
        <div class="text-center mb-8">
          <h1 class="text-2xl font-bold text-text">RustVault</h1>
          <p class="text-sm text-text-secondary mt-1">
            Personal finance, self-hosted
          </p>
        </div>

        {props.children}
      </div>
    </div>
  );
};
