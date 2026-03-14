/**
 * Register page — create a new account.
 */

import { createSignal, Show } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { Button, TextField, showToast } from "~/components/ui";
import { AuthLayout } from "~/components/layout";
import { authStore } from "~/stores";
import { useI18n } from "~/i18n";
import { ApiError } from "~/api/client";

export default function RegisterPage() {
  const t = useI18n();
  const navigate = useNavigate();

  const [username, setUsername] = createSignal("");
  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [confirmPassword, setConfirmPassword] = createSignal("");
  const [error, setError] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError("");

    if (password() !== confirmPassword()) {
      setError("Passwords do not match.");
      return;
    }

    if (password().length < 10) {
      setError("Password must be at least 10 characters.");
      return;
    }

    try {
      await authStore.register(username(), email(), password());
      showToast({
        title: "Account created",
        description: "You can now sign in.",
        variant: "success",
      });
      navigate("/login", { replace: true });
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
          {t("auth.register.title") ?? "Create your account"}
        </h2>

        <Show when={error()}>
          <div class="p-3 rounded-[var(--radius-md)] bg-danger/10 text-danger text-sm">
            {error()}
          </div>
        </Show>

        <TextField
          name="username"
          label={t("auth.register.username") ?? "Username"}
          autocomplete="username"
          value={username()}
          onInput={(e) => setUsername(e.currentTarget.value)}
          required
        />

        <TextField
          name="email"
          label={t("auth.register.email") ?? "Email address"}
          type="email"
          autocomplete="email"
          value={email()}
          onInput={(e) => setEmail(e.currentTarget.value)}
          required
        />

        <TextField
          name="password"
          label={t("auth.register.password") ?? "Password"}
          type="password"
          autocomplete="new-password"
          value={password()}
          onInput={(e) => setPassword(e.currentTarget.value)}
          required
        />

        <TextField
          name="confirm-password"
          label={t("auth.register.confirmPassword") ?? "Confirm password"}
          type="password"
          autocomplete="new-password"
          value={confirmPassword()}
          onInput={(e) => setConfirmPassword(e.currentTarget.value)}
          required
        />

        <Button
          type="submit"
          variant="primary"
          size="md"
          class="w-full"
          loading={authStore.loading()}
        >
          {t("auth.register.submit") ?? "Create account"}
        </Button>

        <p class="text-center text-sm text-text-secondary">
          <A href="/login" class="text-primary hover:underline">
            {t("auth.register.login") ?? "Already have an account? Sign in"}
          </A>
        </p>
      </form>
    </AuthLayout>
  );
}
