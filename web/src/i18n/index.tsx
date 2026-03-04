/**
 * RustVault i18n — Internationalization module
 *
 * Uses @solid-primitives/i18n with lazy locale loading.
 * Each locale's dictionaries are loaded on demand via dynamic imports.
 *
 * Usage:
 *   import { useI18n, useLocale } from "~/i18n";
 *   const t = useI18n();
 *   t("nav.dashboard");
 */

import {
  type Accessor,
  type ParentComponent,
  createContext,
  createMemo,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import * as i18n from "@solid-primitives/i18n";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Supported locale codes — extend this union as new locales are added. */
export type Locale = "en-US";

/** Default locale used at startup and as fallback. */
export const DEFAULT_LOCALE: Locale = "en-US";

/**
 * Raw (nested) dictionary shape — derived from the en-US JSON files so
 * TypeScript knows every valid key at compile time.
 */
export type RawDictionary = {
  common: typeof import("~/locales/en-US/common.json");
  auth: typeof import("~/locales/en-US/auth.json");
};

/** Flattened dictionary with dot-separated keys (e.g. "common.nav.dashboard"). */
export type Dictionary = i18n.Flatten<RawDictionary>;

// ---------------------------------------------------------------------------
// Locale loader
// ---------------------------------------------------------------------------

/**
 * Dynamically imports all namespace JSON files for a given locale and returns
 * a flattened dictionary. New namespaces must be added here when created.
 */
async function fetchDictionary(locale: Locale): Promise<Dictionary> {
  const [common, auth] = await Promise.all([
    import(`~/locales/${locale}/common.json`).then((m) => m.default ?? m),
    import(`~/locales/${locale}/auth.json`).then((m) => m.default ?? m),
  ]);

  const raw: RawDictionary = { common, auth };
  return i18n.flatten(raw) as Dictionary;
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

interface I18nContextValue {
  /** Current locale signal. */
  locale: Accessor<Locale>;
  /** Switch to a different locale (triggers async dictionary load). */
  setLocale: (locale: Locale) => void;
  /** Translator function — returns translated string, or undefined while loading. */
  t: i18n.NullableTranslator<Dictionary>;
}

const I18nContext = createContext<I18nContextValue>();

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/**
 * Wraps the app tree and provides i18n context to all descendants.
 *
 * ```tsx
 * <I18nProvider>
 *   <App />
 * </I18nProvider>
 * ```
 */
export const I18nProvider: ParentComponent = (props) => {
  const [locale, setLocale] = createSignal<Locale>(detectLocale());

  const [dict] = createResource(locale, fetchDictionary, {
    // The en-US dict will be lazily loaded on first render; until then
    // the translator returns keys as-is (see fallback below).
  });

  const memoed = createMemo(() => dict() as Dictionary | undefined);

  const t = i18n.translator(memoed, i18n.resolveTemplate);

  const value: I18nContextValue = {
    locale,
    setLocale,
    t,
  };

  return (
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    <I18nContext.Provider value={value}>{props.children}</I18nContext.Provider>
  );
};

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

/** Returns the translator function `t`. Throws if used outside `<I18nProvider>`. */
export function useI18n(): i18n.NullableTranslator<Dictionary> {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used within <I18nProvider>");
  return ctx.t;
}

/** Returns locale + setLocale. Throws if used outside `<I18nProvider>`. */
export function useLocale() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useLocale must be used within <I18nProvider>");
  return { locale: ctx.locale, setLocale: ctx.setLocale } as const;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** List of all supported locale codes. */
export const SUPPORTED_LOCALES: readonly Locale[] = ["en-US"] as const;

/**
 * Detect initial locale from browser settings, falling back to DEFAULT_LOCALE.
 */
function detectLocale(): Locale {
  if (typeof navigator === "undefined") return DEFAULT_LOCALE;

  for (const lang of navigator.languages) {
    const candidate = lang as Locale;
    if (SUPPORTED_LOCALES.includes(candidate)) return candidate;
  }

  return DEFAULT_LOCALE;
}
