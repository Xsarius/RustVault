/**
 * Login page — email + password sign-in with optional OIDC button.
 */

import { createSignal, Show } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { Button, TextField } from "~/components/ui";
import { AuthLayout } from "~/components/layout";
import { authStore } from "~/stores";
import { useI18n } from "~/i18n";
import { ApiError } from "~/api/client";

export default function LoginPage() {
  const t = useI18n();
  const navigate = useNavigate();

  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
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
      </form>
    </AuthLayout>
  );
}
