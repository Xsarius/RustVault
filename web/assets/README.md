# App Icon & Splash Screen Assets

This directory contains the source assets for generating all required native icon and splash screen sizes.

## Required Files

| File | Size | Purpose |
|------|------|---------|
| `icon-only.png` | 1024×1024 | App icon (no background — transparent) |
| `icon-foreground.png` | 1024×1024 | Android adaptive icon foreground layer |
| `icon-background.png` | 1024×1024 | Android adaptive icon background layer |
| `splash.png` | 2732×2732 | Splash screen (covers largest iPad) |
| `splash-dark.png` | 2732×2732 | Dark mode splash (optional) |

## Generating from the SVG source

The source icon is `icon.svg`. Convert it to the required PNGs using any of:

```bash
# Using sharp-cli
bunx sharp-cli -i icon.svg -o icon-only.png --resize 1024 1024

# Using Inkscape
inkscape --export-png=icon-only.png --export-width=1024 icon.svg

# Using rsvg-convert (librsvg)
rsvg-convert -w 1024 -h 1024 icon.svg -o icon-only.png
```

## Generating native assets

Once the source PNGs are in place, run from the `web/` directory:

```bash
bun run mobile:icons
```

This calls `@capacitor/assets generate` and writes all required sizes to `ios/` and `android/`.

## Design Guidelines

- **Icon**: The vault shield on a minimal background. Follow Apple's Human Interface Guidelines for rounded corners (the OS applies the mask automatically — export a square icon).
- **Splash**: Centered logo on a solid background matching the app's primary color. Keep it simple — it's only visible for ~1s.
- **Adaptive icon** (Android): Place the vault dial in the `foreground` layer centered within the safe zone (66 % of canvas). Use a solid primary-color background.

## Color Reference

| Token | Light | Dark |
|-------|-------|------|
| Background | `#ffffff` | `#0f1117` |
| Primary | `#4f46e5` | `#6366f1` |
