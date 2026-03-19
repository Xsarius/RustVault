/**
 * @capacitor/assets configuration.
 *
 * Source assets live in web/assets/. Run:
 *   bun run mobile:icons
 *
 * to generate all required icon and splash screen sizes for iOS and Android.
 *
 * Required source files:
 *   assets/icon-only.png        — 1024×1024 px, transparent background
 *   assets/icon-foreground.png  — 1024×1024 px, adaptive icon foreground (Android)
 *   assets/icon-background.png  — 1024×1024 px, adaptive icon background (Android)
 *   assets/splash.png           — 2732×2732 px (covers largest iPad)
 *   assets/splash-dark.png      — 2732×2732 px (dark mode variant, optional)
 */

const config = {
  iconBackgroundColor: "#ffffff",
  iconBackgroundColorDark: "#0f1117",
  splashBackgroundColor: "#ffffff",
  splashBackgroundColorDark: "#0f1117",
  logoSizing: 0.8,
  assets: {
    iconOnly: "./assets/icon-only.png",
    iconForeground: "./assets/icon-foreground.png",
    iconBackground: "./assets/icon-background.png",
    splash: "./assets/splash.png",
    splashDark: "./assets/splash-dark.png",
  },
};

module.exports = config;
