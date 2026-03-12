/**
 * Theme store — manages dark/light/system theme preference.
 *
 * Persists choice to localStorage and applies the `dark` class to `<html>`.
 */

import { createSignal, createEffect, createRoot } from "solid-js";

export type Theme = "light" | "dark" | "system";

const STORAGE_KEY = "rustvault-theme";

function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function createThemeStore() {
  const stored = typeof localStorage !== "undefined"
    ? (localStorage.getItem(STORAGE_KEY) as Theme | null)
    : null;

  const [theme, setThemeSignal] = createSignal<Theme>(stored ?? "system");

  function resolvedTheme(): "light" | "dark" {
    const t = theme();
    return t === "system" ? getSystemTheme() : t;
  }

  function setTheme(t: Theme) {
    setThemeSignal(t);
    localStorage.setItem(STORAGE_KEY, t);
  }

  // Apply dark class reactively
  createEffect(() => {
    const resolved = resolvedTheme();
    document.documentElement.classList.toggle("dark", resolved === "dark");
  });

  // Listen for system theme changes when "system" is selected
  if (typeof window !== "undefined") {
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (theme() === "system") {
        // Force re-evaluation by reading the signal
        document.documentElement.classList.toggle("dark", getSystemTheme() === "dark");
      }
    });
  }

  return { theme, setTheme, resolvedTheme };
}

/** Singleton theme store. */
export const themeStore = createRoot(createThemeStore);
