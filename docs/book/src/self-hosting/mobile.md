# Mobile App Setup

This guide explains how to connect the RustVault mobile app (iOS / Android) to your self-hosted backend and how to build the app for distribution.

---

## Overview

The RustVault mobile app is a native wrapper around the same SolidJS web frontend that runs in your browser. It is built with [Capacitor](https://capacitorjs.com/), which packages the compiled web app into a native iOS/Android project.

```
web/dist/  ←  Vite build output
    |
    └──→  Capacitor iOS project  (web/ios/)
    └──→  Capacitor Android project  (web/android/)
```

The mobile app communicates with your self-hosted backend over HTTPS — there is no separate backend for mobile.

---

## Prerequisites

| Tool | Required for |
|------|-------------|
| [Node.js ≥ 20](https://nodejs.org/) | Building the web frontend |
| [Bun](https://bun.sh/) | Package manager (used by this project) |
| [Xcode ≥ 15](https://developer.apple.com/xcode/) | iOS builds (macOS only) |
| [Android Studio](https://developer.android.com/studio) | Android builds |
| [CocoaPods](https://cocoapods.org/) | iOS dependency management (`gem install cocoapods`) |

---

## Initial Setup

### 1. Build the web frontend

```bash
cd web
bun install
bun run build
```

This produces `web/dist/`.

### 2. Add native platforms

```bash
# Inside web/
bunx cap add ios
bunx cap add android
```

This creates `web/ios/` and `web/android/` directories.

### 3. Sync

```bash
bun run mobile:sync
# equivalent to: bun run build && bunx cap sync
```

Run this after **every** frontend change before testing on device.

---

## Connecting to Your Backend

### Production build

Open `web/capacitor.config.ts` and **remove** (or leave unset) the `server.url` field:

```ts
const config: CapacitorConfig = {
  appId: "app.rustvault",
  appName: "RustVault",
  webDir: "dist",
  // No server.url — the app serves from the bundled dist/ files
  // and points to your backend via the VITE_API_BASE_URL at build time.
};
```

The app uses the same origin-relative `/api` paths as the web app. On a native device there is no "same origin", so you must set the backend URL as a build-time environment variable:

```bash
VITE_API_BASE_URL=https://rustvault.yourdomain.com bun run build
bun run mobile:sync
```

Then configure the API client (`web/src/api/client.ts`) to read `import.meta.env.VITE_API_BASE_URL` as the base URL.

### Development / local testing

Set `CAPACITOR_DEV_SERVER_URL` to point to your running Vite server so the native webview loads the hot-reloaded version:

```bash
CAPACITOR_DEV_SERVER_URL=http://192.168.1.100:3000 bunx cap run ios
```

> **Note:** Use your machine's LAN IP, not `localhost` — the device cannot reach the host loop-back interface.

---

## Running on Simulators / Emulators

### iOS simulator

```bash
bun run mobile:run:ios
# or: bun run mobile:ios  (opens Xcode)
```

### Android emulator

```bash
bun run mobile:run:android
# or: bun run mobile:android  (opens Android Studio)
```

---

## TLS / HTTPS Requirement

The backend **must** be served over HTTPS when using the production mobile build, because:

- iOS enforces App Transport Security (ATS) — plain HTTP connections fail by default.
- Biometric auth tokens are stored in the iOS Keychain / Android Keystore; these are only unlocked in secure contexts.

See [docs/book/src/self-hosting/reverse-proxy.md](./reverse-proxy.md) for Nginx / Caddy / Traefik configuration examples with Let's Encrypt.

---

## Biometric Authentication (iOS Face ID / Touch ID, Android Fingerprint)

Biometric unlock is implemented via `@capacitor/preferences`, which is backed by the iOS Keychain and Android Keystore. The auth token is stored securely on the device after the first successful login.

To enable:

1. The user logs in with their username/password or OIDC provider.
2. In **Settings → Security**, toggle **Biometric Unlock**.
3. On subsequent app launches, the user is prompted for biometrics instead of entering credentials.

No additional backend configuration is required.

---

## Camera / Receipt Capture

Receipt capture requires camera permission. The permission dialog is shown the first time the user taps the camera button in the AI-enabled transaction entry flow.

- **iOS**: Add `NSCameraUsageDescription` to `ios/App/App/Info.plist` (Capacitor adds this automatically on `cap sync`).
- **Android**: `CAMERA` permission is declared in the merged `AndroidManifest.xml`.

---

## Push Notifications (Optional)

Budget alert push notifications are optional and disabled by default. To enable:

1. Configure Firebase Cloud Messaging (FCM) on Android or Apple Push Notification Service (APNS) on iOS in your Capacitor project.
2. Set the device push token in the backend via `POST /api/notifications/push-token`.
3. The backend sends alerts when budget thresholds are crossed.

---

## App Icon & Splash Screen

Replace the placeholder assets in:

- `web/ios/App/App/Assets.xcassets/AppIcon.appiconset/` — iOS icons
- `web/android/app/src/main/res/` — Android icons (`mipmap-*` folders)
- `web/ios/App/App/Assets.xcassets/Splash.imageset/` — iOS splash
- `web/android/app/src/main/res/drawable/` — Android splash

Then regenerate with:

```bash
bunx @capacitor/assets generate --iconBackgroundColor '#ffffff' --iconBackgroundColorDark '#0f0f0f' --splashBackgroundColor '#ffffff' --splashBackgroundColorDark '#0f0f0f'
```

---

## Troubleshooting

| Problem | Solution |
|---------|---------|
| `cap sync` fails with pod install error | Run `cd ios/App && pod install --repo-update` |
| "Network request failed" on device | Check that `VITE_API_BASE_URL` is set and backend is HTTPS |
| White screen on launch | Check the browser console in Safari → Develop → [device name] |
| Large bundle size | Run `bun run build` with `VITE_ANALYZE=1` and inspect `rollup-plugin-visualizer` report |
