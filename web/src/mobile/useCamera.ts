/**
 * useCamera — capture a photo via the device camera or photo library.
 *
 * Falls back gracefully on web (uses the browser <input type="file"> picker).
 */

import { Camera, CameraResultType, CameraSource } from "@capacitor/camera";

export interface CapturedPhoto {
  /** Base-64 encoded JPEG data (without data: URI prefix) */
  base64: string;
  /** MIME type, always "image/jpeg" */
  mimeType: "image/jpeg";
}

/**
 * Prompt the user to take a photo or select from the camera roll.
 * Returns null when the user cancels.
 */
async function capturePhoto(
  source: "camera" | "gallery" = "camera",
): Promise<CapturedPhoto | null> {
  try {
    const photo = await Camera.getPhoto({
      quality: 85,
      allowEditing: false,
      resultType: CameraResultType.Base64,
      source:
        source === "camera" ? CameraSource.Camera : CameraSource.Photos,
      correctOrientation: true,
    });

    if (!photo.base64String) return null;

    return {
      base64: photo.base64String,
      mimeType: "image/jpeg",
    };
  } catch (err: unknown) {
    // User cancelled — Capacitor throws "User cancelled photos app" on iOS.
    if (err instanceof Error && /cancel/i.test(err.message)) return null;
    throw err;
  }
}

export function useCamera() {
  return { capturePhoto };
}
