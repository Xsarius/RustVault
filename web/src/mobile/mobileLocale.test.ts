/**
 * mobileLocale — device locale detection tests.
 *
 * Tests the locale resolution logic for both web (navigator.language)
 * and native Capacitor paths.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { Device, resetCapacitorMocks } from "~/test/capacitorMocks";

vi.mock("@capacitor/device", () => ({ Device }));

describe("mobileLocale – web environment (isMobile=false)", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  it("returns navigator.language when set to a known locale", async () => {
    Object.defineProperty(navigator, "language", {
      value: "pl-PL",
      configurable: true,
    });

    const { mobileLocale } = await import("~/mobile/mobileLocale");
    const locale = await mobileLocale();

    // On web isMobile() is false, so it should fall back to navigator.language.
    expect(locale).toBe("pl-PL");
  });

  it("returns 'en-US' when navigator.language is empty", async () => {
    Object.defineProperty(navigator, "language", {
      value: "",
      configurable: true,
    });
    Object.defineProperty(navigator, "languages", {
      value: [],
      configurable: true,
    });

    const { mobileLocale } = await import("~/mobile/mobileLocale");
    const locale = await mobileLocale();
    expect(locale).toBe("en-US");
  });

  it("Device.getLanguageTag is NOT called on web platform", async () => {
    Object.defineProperty(navigator, "language", {
      value: "de-DE",
      configurable: true,
    });

    const { mobileLocale } = await import("~/mobile/mobileLocale");
    await mobileLocale();

    // On web, the Device plugin should not be invoked.
    expect(Device.getLanguageTag).not.toHaveBeenCalled();
  });
});
