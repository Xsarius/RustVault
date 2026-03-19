# Building the Mobile App (Capacitor)

This document describes how contributors can build and test the RustVault iOS and Android apps locally.

---

## Architecture

The mobile apps are Capacitor wrappers around the SolidJS web frontend. There is no separate mobile codebase — changes to `web/src/` affect both the web app and the native apps.

```
web/
├── capacitor.config.ts     ← Capacitor project configuration
├── dist/                   ← Built web assets (served by the native webview)
├── ios/                    ← Xcode project (generated, not committed)
└── android/                ← Gradle project (generated, not committed)
```

The `ios/` and `android/` directories are **generated** and should not be committed to the repository. They are in `.gitignore`.

---

## Quick Start

```bash
cd web

# 1. Install dependencies
bun install

# 2. Build the web frontend
bun run build

# 3. Add platforms (first time only)
bunx cap add ios
bunx cap add android

# 4. Sync web assets to native projects
bunx cap sync

# 5. Run on simulator / emulator
bun run mobile:run:ios      # iOS
bun run mobile:run:android  # Android
```

---

## Available Scripts

| Script | Description |
|--------|-------------|
| `bun run mobile:sync` | Build + sync web assets to native projects |
| `bun run mobile:ios` | Build + open Xcode |
| `bun run mobile:android` | Build + open Android Studio |
| `bun run mobile:run:ios` | Build + run on iOS simulator |
| `bun run mobile:run:android` | Build + run on Android emulator |

---

## Mobile-specific Source Code

Mobile utilities live in `web/src/mobile/`:

| File | Purpose |
|------|---------|
| `useMobile.ts` | Detect native platform (`ios` / `android` / `web`) |
| `useCamera.ts` | Camera capture for receipt scanning |
| `useFilePicker.ts` | Native file picker for importing bank statements |
| `useShareFile.ts` | Native share sheet for exporting data |
| `useBiometric.ts` | Biometric auth + secure token storage |
| `usePushNotifications.ts` | Register & handle push notifications |
| `mobileLocale.ts` | Detect device locale for i18n |

Mobile UI components are in `web/src/components/mobile/`:

| File | Purpose |
|------|---------|
| `PullToRefresh.tsx` | Pull-to-refresh wrapper for list pages |
| `SwipeableRow.tsx` | Swipe-to-reveal action buttons on list rows |

---

## Live Reload During Development

To get hot-module replacement on a physical device or simulator:

```bash
# Start the Vite dev server (must be reachable from device)
bun run dev

# In another terminal, run with the dev server URL:
CAPACITOR_DEV_SERVER_URL=http://192.168.1.100:3000 bunx cap run ios
```

> Use your machine's **LAN IP address** — the device cannot reach `localhost`.

---

## Capacitor Plugins Used

| Plugin | Version | Purpose |
|--------|---------|---------|
| `@capacitor/camera` | 8.x | Receipt photo capture |
| `@capacitor/filesystem` | 8.x | File import from device storage |
| `@capacitor/share` | 8.x | Export via native share sheet |
| `@capacitor/push-notifications` | 8.x | Budget alert push notifications |
| `@capacitor/preferences` | 8.x | Secure key-value storage (auth tokens) |
| `@capacitor/device` | 8.x | Device locale detection |

---

## Adding a New Capacitor Plugin

1. Install the plugin: `bun add @capacitor/plugin-name`
2. Run `bunx cap sync` to link the native module.
3. Add a wrapper in `web/src/mobile/` following the existing pattern.
4. Export from `web/src/mobile/index.ts`.
5. For iOS, run `cd ios/App && pod install` if the plugin has native dependencies.

---

## Testing Mobile-specific Features

Since most CI runs on Linux without Xcode/Android SDK, mobile-specific features are tested via:

1. **Browser touchscreen emulation** in Chrome DevTools (Device Mode).
2. **Capacitor CLI run** on macOS in the iOS simulator (run locally or in a macOS CI job).
3. **Android emulator** in an Android CI job.

The `isMobile()` helper in `useMobile.ts` returns `false` in browsers, so mobile-only code paths (camera, filesystem, biometrics) fall back gracefully and can be tested manually.
