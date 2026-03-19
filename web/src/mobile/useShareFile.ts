/**
 * useShareFile — share a file (e.g., CSV export) via the native share sheet.
 * Falls back to the Web Share API on browsers that support it.
 */

import { Share } from "@capacitor/share";
import { Filesystem, Directory } from "@capacitor/filesystem";
import { isMobile } from "./useMobile";

export interface ShareFileOptions {
  /** File name including extension */
  filename: string;
  /** Base-64 encoded file contents */
  base64: string;
  mimeType: string;
  /** Share dialog title (Android) */
  dialogTitle?: string;
}

/**
 * Write a file to a temporary directory and open the native share sheet.
 */
async function shareFile(opts: ShareFileOptions): Promise<void> {
  const { filename, base64, mimeType, dialogTitle } = opts;

  if (isMobile()) {
    // Write to cache directory so the native share sheet can access it.
    const writeResult = await Filesystem.writeFile({
      path: filename,
      data: base64,
      directory: Directory.Cache,
    });

    await Share.share({
      title: filename,
      url: writeResult.uri,
      dialogTitle: dialogTitle ?? filename,
    });

    // Clean up after sharing.
    await Filesystem.deleteFile({
      path: filename,
      directory: Directory.Cache,
    }).catch(() => {
      // Ignore cleanup errors.
    });
  } else if ("share" in navigator) {
    // Web Share API fallback.
    const blob = base64ToBlob(base64, mimeType);
    const file = new File([blob], filename, { type: mimeType });
    await navigator.share({ files: [file], title: filename });
  } else {
    // Last resort: trigger a browser download.
    const blob = base64ToBlob(base64, mimeType);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
  }
}

function base64ToBlob(base64: string, mimeType: string): Blob {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new Blob([bytes], { type: mimeType });
}

export function useShareFile() {
  return { shareFile };
}
