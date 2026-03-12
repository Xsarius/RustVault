/**
 * Settings page — tabbed layout for General, Appearance, Account settings.
 */

import { createSignal } from "solid-js";
import {
  Tabs,
  TabList,
  TabTrigger,
  TabContent,
  Button,
  TextField,
  Select,
  showToast,
} from "~/components/ui";
import { themeStore, authStore } from "~/stores";
import { useI18n } from "~/i18n";
import type { Theme } from "~/stores/theme";

export default function SettingsPage() {
  const t = useI18n();

  return (
    <div class="space-y-6 max-w-2xl">
      <h1 class="text-2xl font-bold text-text">
        {t("common.nav.settings") ?? "Settings"}
      </h1>

      <Tabs defaultValue="general">
        <TabList>
          <TabTrigger value="general">General</TabTrigger>
          <TabTrigger value="appearance">Appearance</TabTrigger>
          <TabTrigger value="account">Account</TabTrigger>
        </TabList>

        <TabContent value="general">
          <GeneralSettings />
        </TabContent>
        <TabContent value="appearance">
          <AppearanceSettings />
        </TabContent>
        <TabContent value="account">
          <AccountSettings />
        </TabContent>
      </Tabs>
    </div>
  );
}

// ── General ──────────────────────────────────────────────────

function GeneralSettings() {
  const [currency, setCurrency] = createSignal("USD");
  const [locale, setLocale] = createSignal("en-US");

  const handleSave = () => {
    // TODO: Persist settings via API
    showToast({ title: "Settings saved", variant: "success" });
  };

  return (
    <div class="space-y-6 py-4">
      <SettingsSection title="Defaults">
        <Select
          name="currency"
          label="Default currency"
          options={[
            { value: "USD", label: "USD — US Dollar" },
            { value: "EUR", label: "EUR — Euro" },
            { value: "GBP", label: "GBP — British Pound" },
            { value: "PLN", label: "PLN — Polish Złoty" },
            { value: "CHF", label: "CHF — Swiss Franc" },
            { value: "JPY", label: "JPY — Japanese Yen" },
          ]}
          value={currency()}
          onChange={setCurrency}
        />
        <Select
          name="locale"
          label="Language"
          options={[{ value: "en-US", label: "English (US)" }]}
          value={locale()}
          onChange={setLocale}
        />
      </SettingsSection>

      <div class="flex justify-end">
        <Button variant="primary" size="sm" onClick={handleSave}>
          Save Changes
        </Button>
      </div>
    </div>
  );
}

// ── Appearance ───────────────────────────────────────────────

function AppearanceSettings() {
  const { theme, setTheme } = themeStore;

  return (
    <div class="space-y-6 py-4">
      <SettingsSection title="Theme">
        <div class="grid grid-cols-3 gap-3">
          <ThemeOption
            label="Light"
            value="light"
            current={theme()}
            onSelect={setTheme}
          />
          <ThemeOption
            label="Dark"
            value="dark"
            current={theme()}
            onSelect={setTheme}
          />
          <ThemeOption
            label="System"
            value="system"
            current={theme()}
            onSelect={setTheme}
          />
        </div>
      </SettingsSection>
    </div>
  );
}

function ThemeOption(props: {
  label: string;
  value: Theme;
  current: Theme;
  onSelect: (t: Theme) => void;
}) {
  const selected = () => props.current === props.value;

  return (
    <button
      class={`flex flex-col items-center gap-2 p-4 rounded-[var(--radius-lg)] border-2 transition-colors cursor-pointer ${
        selected()
          ? "border-primary bg-primary/5"
          : "border-border hover:border-text-tertiary"
      }`}
      onClick={() => props.onSelect(props.value)}
    >
      {/* Preview swatch */}
      <div
        class={`w-full h-16 rounded-[var(--radius-md)] ${
          props.value === "dark"
            ? "bg-[#09090B]"
            : props.value === "light"
              ? "bg-white border border-gray-200"
              : "bg-gradient-to-r from-white to-[#09090B]"
        }`}
      />
      <span class="text-sm font-medium text-text">{props.label}</span>
    </button>
  );
}

// ── Account ──────────────────────────────────────────────────

function AccountSettings() {
  const user = authStore.user;

  return (
    <div class="space-y-6 py-4">
      <SettingsSection title="Profile">
        <TextField
          name="username"
          label="Username"
          value={user()?.username ?? ""}
          disabled
        />
        <TextField
          name="email"
          label="Email"
          type="email"
          value={user()?.email ?? ""}
          disabled
        />
      </SettingsSection>

      <SettingsSection title="Security">
        <Button variant="secondary" size="sm">
          Change Password
        </Button>
      </SettingsSection>

      <SettingsSection title="Danger Zone">
        <div class="p-4 rounded-[var(--radius-md)] border border-danger/30 bg-danger/5">
          <p class="text-sm text-text-secondary mb-3">
            Permanently delete your account and all associated data. This action cannot be undone.
          </p>
          <Button variant="danger" size="sm">
            Delete Account
          </Button>
        </div>
      </SettingsSection>
    </div>
  );
}

// ── Helpers ──────────────────────────────────────────────────

function SettingsSection(props: { title: string; children: any }) {
  return (
    <div class="space-y-4">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wide">
        {props.title}
      </h3>
      {props.children}
    </div>
  );
}
