/**
 * ServerSetup — first-run screen for the mobile app.
 *
 * Shown before login when running inside Capacitor and no server URL has
 * been saved yet. The user types the base URL of their self-hosted
 * RustVault instance (e.g. https://vault.example.com) and taps "Connect".
 *
 * On web this page is never shown (relative API paths always work).
 */

import { createSignal, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Server, CheckCircle, AlertCircle, Loader } from "lucide-solid";
import { serverStore } from "~/stores";
import { useI18n } from "~/i18n";

type TestState = "idle" | "testing" | "ok" | "error";

export default function ServerSetupPage() {
  const t = useI18n();
  const navigate = useNavigate();

  const [url, setUrl] = createSignal(serverStore.serverUrl() || "https://");
  const [testState, setTestState] = createSignal<TestState>("idle");
  const [testError, setTestError] = createSignal("");
  const [saving, setSaving] = createSignal(false);

  /** Normalise the URL: trim whitespace and strip trailing slash. */
  function normalise(raw: string): string {
    return raw.trim().replace(/\/+$/, "");
  }

  function isValidUrl(raw: string): boolean {
    try {
      const u = new URL(normalise(raw));
      return u.protocol === "http:" || u.protocol === "https:";
    } catch {
      return false;
    }
  }

  /** Hit /api/health to verify the server is reachable. */
  async function testConnection() {
    if (!isValidUrl(url())) {
      setTestState("error");
      setTestError(t("auth.serverSetup.invalidUrl") ?? "Please enter a valid URL.");
      return;
    }

    setTestState("testing");
    setTestError("");

    try {
      const res = await fetch(`${normalise(url())}/api/health`, {
        method: "GET",
        signal: AbortSignal.timeout(8000),
      });
      if (res.ok || res.status === 404) {
        // 404 means the server responded — health endpoint may not exist yet.
        setTestState("ok");
      } else {
        setTestState("error");
        setTestError(
          t("auth.serverSetup.testFailed") ??
            `Server responded with status ${res.status}.`,
        );
      }
    } catch {
      setTestState("error");
      setTestError(
        t("auth.serverSetup.unreachable") ??
          "Could not reach the server. Check the URL and your network connection.",
      );
    }
  }

  async function handleConnect() {
    if (!isValidUrl(url())) {
      setTestState("error");
      setTestError(t("auth.serverSetup.invalidUrl") ?? "Please enter a valid URL.");
      return;
    }

    setSaving(true);
    try {
      await serverStore.setServerUrl(normalise(url()));
      navigate("/login", { replace: true });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div class="min-h-screen bg-background flex flex-col items-center justify-center p-6 safe-area-pt safe-area-pb">
      <div class="w-full max-w-sm space-y-8">
        {/* Icon + heading */}
        <div class="text-center space-y-3">
          <div class="inline-flex items-center justify-center h-16 w-16 rounded-2xl bg-primary/10 text-primary mx-auto">
            <Server size={32} />
          </div>
          <h1 class="text-2xl font-bold text-text">
            {t("auth.serverSetup.title") ?? "Connect to your server"}
          </h1>
          <p class="text-sm text-text-secondary">
            {t("auth.serverSetup.subtitle") ??
              "Enter the URL of your self-hosted RustVault instance."}
          </p>
        </div>

        {/* URL input */}
        <div class="space-y-2">
          <label class="block text-sm font-medium text-text" for="server-url">
            {t("auth.serverSetup.urlLabel") ?? "Server URL"}
          </label>
          <input
            id="server-url"
            type="url"
            inputmode="url"
            autocomplete="url"
            spellcheck={false}
            autocorrect="off"
            autocapitalize="none"
            value={url()}
            onInput={(e) => {
              setUrl(e.currentTarget.value);
              setTestState("idle");
            }}
            onKeyDown={(e) => e.key === "Enter" && handleConnect()}
            placeholder="https://rustvault.example.com"
            class="w-full px-3 py-2.5 rounded-[var(--radius-md)] border border-border bg-surface text-text placeholder:text-text-tertiary text-base focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
          />

          {/* Test result indicator */}
          <Show when={testState() !== "idle"}>
            <div
              class={`flex items-center gap-2 text-sm mt-1 ${
                testState() === "ok"
                  ? "text-income"
                  : testState() === "error"
                    ? "text-danger"
                    : "text-text-secondary"
              }`}
            >
              <Show when={testState() === "testing"}>
                <Loader size={14} class="animate-spin" />
                <span>{t("auth.serverSetup.testing") ?? "Testing connection…"}</span>
              </Show>
              <Show when={testState() === "ok"}>
                <CheckCircle size={14} />
                <span>{t("auth.serverSetup.testOk") ?? "Server reachable!"}</span>
              </Show>
              <Show when={testState() === "error"}>
                <AlertCircle size={14} />
                <span>{testError()}</span>
              </Show>
            </div>
          </Show>
        </div>

        {/* Actions */}
        <div class="space-y-3">
          <button
            type="button"
            disabled={saving()}
            onClick={handleConnect}
            class="w-full flex items-center justify-center gap-2 py-2.5 px-4 rounded-[var(--radius-md)] bg-primary text-white text-sm font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer"
          >
            <Show when={saving()}>
              <Loader size={16} class="animate-spin" />
            </Show>
            {t("auth.serverSetup.connect") ?? "Connect"}
          </button>

          <button
            type="button"
            disabled={testState() === "testing" || saving()}
            onClick={testConnection}
            class="w-full py-2.5 px-4 rounded-[var(--radius-md)] border border-border text-text-secondary text-sm font-medium hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer"
          >
            {t("auth.serverSetup.testConnection") ?? "Test connection"}
          </button>
        </div>
      </div>
    </div>
  );
}
