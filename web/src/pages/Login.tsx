/**
 * Login page — email + password sign-in with optional OIDC button.
 */

import { createSignal, Show } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { Button, TextField } from "~/components/ui";
import { AuthLayout } from "~/components/layout";
import { authStore, serverStore } from "~/stores";
import { isMobile } from "~/mobile/useMobile";
import { useI18n } from "~/i18n";
import { ApiError } from "~/api/client";

declare const __DEMO_MODE__: boolean;

const DEMO_EMAIL = "demo@rustvault.app";
const DEMO_PASSWORD = "demo";

export default function LoginPage() {
  const t = useI18n();
  const navigate = useNavigate();

  const [email, setEmail] = createSignal(__DEMO_MODE__ ? DEMO_EMAIL : "");
  const [password, setPassword] = createSignal(__DEMO_MODE__ ? DEMO_PASSWORD : "");
  const [error, setError] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError("");

    try {
      await authStore.login(email(), password());
      navigate("/", { replace: true });
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.message);
      } else {
        setError("An unexpected error occurred.");
      }
    }
  };

  return (
    <AuthLayout>
      <form onSubmit={handleSubmit} class="space-y-4">
        <h2 class="text-lg font-semibold text-text text-center">
          {t("auth.login.title") ?? "Sign in to RustVault"}
        </h2>

        <Show when={__DEMO_MODE__}>
          <div class="p-3 rounded-[var(--radius-md)] text-sm" style={{ background: "rgba(217,119,6,0.10)", border: "1px solid rgba(217,119,6,0.35)" }}>
            <p class="font-semibold mb-1" style={{ color: "#92400e" }}>Demo mode — use these credentials:</p>
            <div class="font-mono text-xs space-y-0.5" style={{ color: "#78350f" }}>
              <p>Email: <strong>{DEMO_EMAIL}</strong></p>
              <p>Password: <strong>{DEMO_PASSWORD}</strong></p>
            </div>
            <p class="mt-1 text-xs" style={{ color: "#a16207" }}>Fields are pre-filled. Just click <em>Sign in</em>.</p>
          </div>
        </Show>

        <Show when={error()}>
          <div class="p-3 rounded-[var(--radius-md)] bg-danger/10 text-danger text-sm">
            {error()}
          </div>
        </Show>

        <TextField
          name="email"
          label={t("auth.login.email") ?? "Email address"}
          type="email"
          autocomplete="username"
          value={email()}
          onInput={(e) => setEmail(e.currentTarget.value)}
          required
        />

        <TextField
          name="password"
          label={t("auth.login.password") ?? "Password"}
          type="password"
          autocomplete="current-password"
          value={password()}
          onInput={(e) => setPassword(e.currentTarget.value)}
          required
        />

        <Button
          type="submit"
          variant="primary"
          size="md"
          class="w-full"
          loading={authStore.loading()}
        >
          {t("auth.login.submit") ?? "Sign in"}
        </Button>

        <p class="text-center text-sm text-text-secondary">
          <A href="/register" class="text-primary hover:underline">
            {t("auth.register.login")
              ? t("auth.login.register")
              : "Create an account"}
          </A>
        </p>

        {/* On mobile, allow switching the server without going through Settings */}
        <Show when={isMobile()}>
          <p class="text-center text-xs text-text-tertiary">
            <A href="/server-setup" class="hover:underline">
              {t("auth.login.changeServer") ?? serverStore.serverUrl() ?? "Change server"}
            </A>
          </p>
        </Show>
      </form>
    </AuthLayout>
  );
}
