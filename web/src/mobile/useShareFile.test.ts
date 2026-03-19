import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  Share,
  Filesystem,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

// Always run as the web platform so the non-native branches are exercised.
vi.mock("@capacitor/core", () => ({
  Capacitor: {
    getPlatform: vi.fn(() => "web"),
    isNativePlatform: vi.fn(() => false),
  },
}));
vi.mock("@capacitor/share", () => ({ Share }));
vi.mock("@capacitor/filesystem", () => ({
  Filesystem,
  Directory: { Cache: "CACHE" },
}));

const { useShareFile } = await import("~/mobile/useShareFile");
const { shareFile } = useShareFile();

const OPTS = {
  filename: "export.csv",
  base64: "dGVzdA==", // "test"
  mimeType: "text/csv",
};

describe("useShareFile (web platform)", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  describe("Web Share API path", () => {
    beforeEach(() => {
      // jsdom doesn't implement navigator.share — define it so vi.spyOn can
      // wrap it, then assert correctly.
      Object.defineProperty(navigator, "share", {
        value: vi.fn().mockResolvedValue(undefined),
        configurable: true,
        writable: true,
      });
    });

    afterEach(() => {
      // @ts-expect-error intentional removal to restore jsdom default state
      delete (navigator as Navigator & { share?: unknown }).share;
    });

    it("calls navigator.share with a File when available", async () => {
      await shareFile(OPTS);
      expect(navigator.share).toHaveBeenCalledOnce();
      const callArgs = (navigator.share as ReturnType<typeof vi.fn>).mock
        .calls[0][0] as ShareData;
      expect(callArgs.title).toBe("export.csv");
      expect((callArgs.files as File[])[0].name).toBe("export.csv");
    });
  });

  describe("download fallback path", () => {
    let origShare: typeof navigator.share;
    let anchorClickSpy: ReturnType<typeof vi.spyOn>;
    let createSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
      // Remove navigator.share to force the download fallback.
      origShare = navigator.share;
      // @ts-expect-error intentional removal
      delete navigator.share;

      const origCreate = document.createElement.bind(document);
      createSpy = vi.spyOn(document, "createElement").mockImplementation(
        (tag: string) => {
          const el = origCreate(tag);
          if (tag === "a") {
            anchorClickSpy = vi.spyOn(el as HTMLAnchorElement, "click").mockImplementation(() => {});
          }
          return el;
        },
      );

      vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:fake");
      vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});
    });

    afterEach(() => {
      // Restore navigator.share
      Object.defineProperty(navigator, "share", {
        value: origShare,
        configurable: true,
        writable: true,
      });
      createSpy.mockRestore();
    });

    it("triggers a browser download when Web Share API is absent", async () => {
      await shareFile(OPTS);
      expect(URL.createObjectURL).toHaveBeenCalledOnce();
      expect(anchorClickSpy!).toHaveBeenCalledOnce();
    });
  });
});
