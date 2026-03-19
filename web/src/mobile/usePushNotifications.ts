/**
 * usePushNotifications — register for push notifications and handle
 * incoming budget alert messages.
 *
 * Gracefully no-ops on web (browser push requires a Service Worker
 * and is handled separately).
 */

import {
  PushNotifications,
  type Token,
  type PushNotificationSchema,
  type ActionPerformed,
} from "@capacitor/push-notifications";
import { isMobile } from "./useMobile";

/** Register the device for push notifications and return the FCM/APNS token. */
async function registerPush(
  onToken: (token: string) => void,
  onNotification: (notification: PushNotificationSchema) => void,
): Promise<void> {
  if (!isMobile()) return;

  const permStatus = await PushNotifications.checkPermissions();

  let resolved = permStatus.receive;
  if (resolved === "prompt") {
    const req = await PushNotifications.requestPermissions();
    resolved = req.receive;
  }

  if (resolved !== "granted") return;

  await PushNotifications.register();

  // Token received (FCM on Android, APNS on iOS).
  await PushNotifications.addListener("registration", (token: Token) => {
    onToken(token.value);
  });

  // Foreground notification (in-app display).
  await PushNotifications.addListener(
    "pushNotificationReceived",
    (notification: PushNotificationSchema) => {
      onNotification(notification);
    },
  );

  // Tapped notification (user opened from notification tray).
  await PushNotifications.addListener(
    "pushNotificationActionPerformed",
    (_action: ActionPerformed) => {
      // Navigation handled by the caller via deep link.
    },
  );
}

/** Remove all listeners (call on logout). */
async function unregisterPush(): Promise<void> {
  if (!isMobile()) return;
  await PushNotifications.removeAllListeners();
}

export function usePushNotifications() {
  return { registerPush, unregisterPush };
}
