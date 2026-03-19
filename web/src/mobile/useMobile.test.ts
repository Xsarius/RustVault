import { describe, it, expect, vi } from "vitest";

vi.mock("@capacitor/core", () => ({
  Capacitor: {
    getPlatform: vi.fn(() => "web"),
    isNativePlatform: vi.fn(() => false),
  },
}));

const { isPlatform, isMobile, useMobile } = await import("~/mobile/useMobile");

describe("useMobile", () => {
  it("isPlatform returns 'web' in jsdom", () => {
    expect(isPlatform()).toBe("web");
  });

  it("isMobile returns false in jsdom", () => {
    expect(isMobile()).toBe(false);
  });

  it("useMobile returns correct shape for web platform", () => {
    const info = useMobile();
    expect(info.platform).toBe("web");
    expect(info.isNative).toBe(false);
    expect(info.isWeb).toBe(true);
    expect(info.isIos).toBe(false);
    expect(info.isAndroid).toBe(false);
  });
});
