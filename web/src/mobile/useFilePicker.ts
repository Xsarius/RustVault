/**
 * useFilePicker — open the native file picker to import a bank statement.
 *
 * On native platforms we use a native document picker (triggered via the
 * Capacitor plugin bridge). On web we fall back to a hidden
 * <input type="file"> element.
 */

import { isMobile } from "./useMobile";

export interface PickedFile {
  name: string;
  /** Base-64 encoded file contents */
  base64: string;
  mimeType: string;
  size: number;
}

/** Maximum allowed file size (50 MB) */
const MAX_BYTES = 50 * 1024 * 1024;

/**
 * Open the native/browser file picker filtered to common bank statement formats.
 * Returns null when the user cancels.
 */
async function pickFile(): Promise<PickedFile | null> {
  if (isMobile()) {
    return pickFileNative();
  }
  return pickFileWeb();
}

/** Native file pick via hidden input bridge (works in Capacitor WKWebView) */
async function pickFileNative(): Promise<PickedFile | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".csv,.ofx,.qfx,.mt940,.camt,.xml,.xlsx,.json,.qif";

    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      if (file.size > MAX_BYTES) {
        resolve(null);
        return;
      }
      const base64 = await fileToBase64(file);
      resolve({ name: file.name, base64, mimeType: file.type, size: file.size });
    };

    input.oncancel = () => resolve(null);
    input.click();
  });
}

/** Browser file pick — identical logic, same as native bridge */
async function pickFileWeb(): Promise<PickedFile | null> {
  return pickFileNative();
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Strip the data URI prefix to get only the base64 payload.
      resolve(result.split(",")[1] ?? result);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

export function useFilePicker() {
  return { pickFile };
}
