# RustVault — UI Wireframes

This directory contains interactive wireframes for RustVault's UI, created with [Excalidraw](https://excalidraw.com/).

## How to view & edit

1. **VS Code:** Install the [Excalidraw extension](https://marketplace.visualstudio.com/items?itemName=pomdtr.excalidraw-editor) — `.excalidraw` files open directly in the editor.
2. **Browser:** Open [excalidraw.com](https://excalidraw.com/) and drag any `.excalidraw` file onto it.
3. **GitHub:** Excalidraw files render as JSON on GitHub. Use the VS Code extension or Excalidraw web app for visual editing.

## Wireframe index

| File | Section | Description |
|------|---------|-------------|
| `shell-layout.excalidraw` | §4.1 | App shell: top bar + sidebar + content area |
| `mobile-layout.excalidraw` | §4.3 | Mobile: top bar + content + bottom tab bar |
| `sidebar.excalidraw` | §4.4 | Sidebar expanded (220px) and collapsed (56px) states |
| `dashboard.excalidraw` | §6.1 | Portfolio overview: net worth, accounts, budget bars |
| `transaction-list.excalidraw` | §6.2 | Desktop table + mobile list views |
| `transaction-detail.excalidraw` | §6.3 | Side panel (desktop) / full-screen (mobile) |
| `import-wizard.excalidraw` | §6.4 | 4-step wizard: Upload → Configure → Preview → Done |
| `banks-accounts.excalidraw` | §6.5 | Bank cards with nested account rows |
| `categories.excalidraw` | §6.6 | Tree view with collapsible nodes |
| `budget.excalidraw` | §6.7 | Budget table with progress bars |
| `reports.excalidraw` | §6.8 | Report tabs with chart + filter bar |
| `settings.excalidraw` | §6.9 | Settings tabs: General, Appearance, Account, AI |
| `rules.excalidraw` | §6.10 | Rule list + rule editor dialog |
| `tags.excalidraw` | §6.11 | Tag cloud page |

## Conventions

- **Colors:** Use the design system palette from UI_PLAN.md §3.1
- **Fonts:** Use Inter for UI text, monospace for amounts
- **Grid:** 8px base grid — align elements to multiples of 8
- **Style:** Hand-drawn mode OFF — use architect style for cleaner wireframes
- **Annotations:** Use Excalidraw text elements with `→` arrows for callouts
- **Breakpoints:** Create separate frames for desktop (1280px) and mobile (375px) views

## Export for GitHub

To make wireframes visible on GitHub without the Excalidraw extension:

```bash
# Export each .excalidraw file to SVG (use Excalidraw CLI or web app)
# Place SVGs alongside .excalidraw files as .svg
# Reference in markdown: ![Dashboard](docs/wireframes/dashboard.svg)
```
