import type { CapacitorConfig } from "@capacitor/cli";

const config: CapacitorConfig = {
  appId: "app.rustvault",
  appName: "RustVault",
  webDir: "dist",
  // Server config is only used during development — point to the local Vite dev server.
  // In production builds the native webview serves the bundled dist/ directly.
  server: {
    url: process.env.CAPACITOR_DEV_SERVER_URL,
    cleartext: true,
  },
  plugins: {
    Camera: {
      // Request permissions on first use, not at startup.
    },
    Filesystem: {},
    Share: {},
    PushNotifications: {
      presentationOptions: ["badge", "sound", "alert"],
    },
    Preferences: {},
    // SplashScreen plugin — controls the native splash behaviour.
    SplashScreen: {
      launchShowDuration: 1500,
      launchAutoHide: true,
      launchFadeOutDuration: 300,
      backgroundColor: "#ffffff",
      androidSplashResourceName: "splash",
      androidScaleType: "CENTER_CROP",
      showSpinner: false,
      splashFullScreen: true,
      splashImmersive: true,
    },
  },
  ios: {
    contentInset: "automatic",
    backgroundColor: "#ffffff",
    allowsLinkPreview: false,
  },
  android: {
    allowMixedContent: false,
    backgroundColor: "#ffffff",
    captureInput: true,
  },
};

export default config;
