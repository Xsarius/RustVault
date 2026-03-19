import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Capacitor is not available in jsdom — expose it as the web platform.
vi.mock("@capacitor/core", () => ({
  Capacitor: {
    getPlatform: vi.fn(() => "web"),
    isNativePlatform: vi.fn(() => false),
  },
}));

const { useFilePicker } = await import("~/mobile/useFilePicker");
const { pickFile } = useFilePicker();

/**
 * jsdom does not implement DataTransfer, so we build a minimal FileList-like
 * object that satisfies the `input.files?.[0]` access pattern in the handler.
 */
function fakeFileList(...files: File[]): FileList {
  const list: FileList = Object.assign([...files], {
    item: (i: number): File | null => files[i] ?? null,
    [Symbol.iterator]: function* () { yield* files; },
  }) as unknown as FileList;
  return list;
}

describe("useFilePicker", () => {
  let createSpy: ReturnType<typeof vi.spyOn>;
  let capturedInput: HTMLInputElement | null = null;

  beforeEach(() => {
    capturedInput = null;
    const origCreate = document.createElement.bind(document);
    createSpy = vi.spyOn(document, "createElement").mockImplementation(
      (tagName: string) => {
        const el = origCreate(tagName);
        if (tagName === "input") {
          capturedInput = el as HTMLInputElement;
          // Prevent real click from opening OS dialogs.
          vi.spyOn(el, "click").mockImplementation(() => {});
        }
        return el;
      },
    );
  });

  afterEach(() => {
    createSpy.mockRestore();
  });

  it("returns null when user cancels", async () => {
    const promise = pickFile();
    capturedInput!.oncancel!(new Event("cancel"));
    expect(await promise).toBeNull();
  });

  it("returns null when no file is attached (empty change event)", async () => {
    const promise = pickFile();
    const event = new Event("change");
    // files is undefined — handler checks `input.files?.[0]` which is falsy.
    await (capturedInput!.onchange as (e: Event) => Promise<void>)(event);
    expect(await promise).toBeNull();
  });

  it("returns null when selected file exceeds 50 MB", async () => {
    const promise = pickFile();

    const bigFile = new File(["x"], "huge.csv", { type: "text/csv" });
    Object.defineProperty(bigFile, "size", { value: 60 * 1024 * 1024 });
    Object.defineProperty(capturedInput!, "files", {
      value: fakeFileList(bigFile),
      configurable: true,
    });

    const event = new Event("change");
    await (capturedInput!.onchange as (e: Event) => Promise<void>)(event);
    expect(await promise).toBeNull();
  });

  it("returns a PickedFile for a valid small file", async () => {
    const promise = pickFile();

    // Mock FileReader to return a predictable base64 string.
    const MockFileReader = vi.fn().mockImplementation(function (this: {
      result: string;
      onload: (() => void) | null;
      onerror: (() => void) | null;
      readAsDataURL: (f: File) => void;
    }) {
      this.result = "data:text/csv;base64,dGVzdA==";
      this.onload = null;
      this.onerror = null;
      this.readAsDataURL = vi.fn(() => {
        // Fire onload in the next microtask.
        Promise.resolve().then(() => this.onload?.());
      });
    });
    const origFileReader = globalThis.FileReader;
    // @ts-expect-error intentional override for testing
    globalThis.FileReader = MockFileReader;

    const file = new File(["test"], "export.csv", { type: "text/csv" });
    Object.defineProperty(capturedInput!, "files", {
      value: fakeFileList(file),
      configurable: true,
    });

    const event = new Event("change");
    const handlerDone = (
      capturedInput!.onchange as (e: Event) => Promise<void>
    )(event);

    await handlerDone;
    const result = await promise;

    globalThis.FileReader = origFileReader;

    expect(result).not.toBeNull();
    expect(result!.name).toBe("export.csv");
    expect(result!.mimeType).toBe("text/csv");
    expect(result!.base64).toBe("dGVzdA==");
  });
});
