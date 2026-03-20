/**
 * useShareFile — real-world scenario tests.
 *
 * Exercises file sharing and cleanup.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  Share,
  Filesystem,
  resetCapacitorMocks,
} from "~/test/capacitorMocks";

// Force isMobile() = true so the Capacitor code path is exercised.
vi.mock("~/mobile/useMobile", () => ({
  isMobile: () => true,
  isPlatform: (_p: string) => true,
  useMobile: () => ({ isMobileDevice: () => true }),
}));
vi.mock("@capacitor/share",      () => ({ Share }));
vi.mock("@capacitor/filesystem", () => ({ Filesystem, Directory: { Cache: "CACHE" } }));

const { useShareFile } = await import("~/mobile/useShareFile");

describe("useShareFile – scenarios", () => {
  beforeEach(() => {
    resetCapacitorMocks();
  });

  it("shareFile writes to filesystem then invokes Share.share", async () => {
    const { shareFile } = useShareFile();
    await shareFile({
      filename: "transactions.csv",
      base64: btoa("date,amount\n2026-03-01,-10.00\n"),
      mimeType: "text/csv",
    });

    expect(Filesystem.writeFile).toHaveBeenCalledTimes(1);
    expect(Share.share).toHaveBeenCalledTimes(1);
  });

  it("shareFile includes filename in the share payload", async () => {
    const { shareFile } = useShareFile();
    await shareFile({
      filename: "report.json",
      base64: btoa(JSON.stringify({ data: [] })),
      mimeType: "application/json",
    });

    const call = (Share.share as ReturnType<typeof vi.fn>).mock.calls[0][0] as Record<string, unknown>;
    expect(typeof call.title === "string").toBe(true);
    expect(call.title).toBe("report.json");
  });

  it("shareFile cleans up the temporary file after sharing", async () => {
    const { shareFile } = useShareFile();
    await shareFile({
      filename: "report.json",
      base64: btoa("{}"),
      mimeType: "application/json",
    });

    expect(Filesystem.deleteFile).toHaveBeenCalledTimes(1);
  });

  it("shareFile propagates error when Share.share rejects", async () => {
    (Share.share as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error("User cancelled"));
    const { shareFile } = useShareFile();

    await expect(
      shareFile({ filename: "export.csv", base64: btoa("a,b\n1,2\n"), mimeType: "text/csv" }),
    ).rejects.toThrow("User cancelled");
  });
});
