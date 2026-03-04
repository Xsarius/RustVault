# ADR-0002: Frontend Architecture

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** frontend, solidjs, typescript, capacitor

## Context

RustVault needs a frontend that:

- Renders large transaction tables (10k+ rows) without jank
- Ships a small bundle — self-hosted users may run on constrained hardware
- Supports reactive forms with complex validation (import mapping, budget editing)
- Wraps cleanly into native iOS/Android apps
- Has a mature component ecosystem for finance UIs (tables, charts, headless components)

## Decisions

### Framework: SolidJS + TypeScript

SolidJS uses fine-grained reactivity (signals) instead of virtual DOM diffing. Only specific DOM nodes update when data changes — no component re-renders. Its JSX syntax is familiar to React developers.

| Alternative | Why not |
|-------------|---------|
| **React** | Virtual DOM overhead for large lists; larger bundle; hooks re-render entire components |
| **Svelte** | Smaller TypeScript ecosystem; component libraries less mature |
| **Vue 3** | Heavier bundle; template syntax diverges from JSX; less granular reactivity |
| **Leptos (Rust/WASM)** | Immature ecosystem; limited component libraries; WASM bundle size |

### Component Ecosystem

| Concern | Choice | Role |
|---------|--------|------|
| Headless UI | **Kobalte** | Accessible primitives (dialog, select, tabs, combobox) |
| Styling | **Tailwind CSS** | Utility-first, no runtime overhead |
| Data table | **@tanstack/solid-table** + **solid-virtual** | Headless, sortable, filterable, virtualized rows |
| Forms | **@modular-forms/solid** | Type-safe reactive form management |
| Charts | **Apache ECharts** | Modular import, lazy-loaded, financial-grade visualizations |
| Icons | **Lucide** (lucide-solid) | MIT, tree-shakeable, 1000+ icons |
| Typography | **Inter** + **JetBrains Mono** | UI clarity + monospace digit alignment for finance |

### Mobile: Capacitor

Capacitor wraps the SolidJS SPA into native iOS/Android containers. No SSR complexity — the same build output works for web and mobile.

### Build: Vite

Sub-second HMR, native ESM dev server, optimized production builds with code splitting and tree shaking.

## Consequences

### Positive

- Fine-grained reactivity — only changed DOM nodes update; no component re-renders
- Tiny bundle — SolidJS core is ~7KB gzipped
- Familiar JSX — React developers onboard quickly
- Standard tooling (Vite, ESLint, Prettier) works out of the box
- Capacitor-ready SPA output — no SSR needed
- Kobalte provides WCAG-compliant headless components

### Negative

- Smaller community than React — fewer tutorials and third-party components
- Some React libraries don't have Solid ports (mitigated: headless/vanilla-JS libraries work)
- Signal/memo/effect model differs from React hooks — learning curve for React developers

### Risks

- SolidJS adoption could stall (mitigated: actively maintained, SolidStart gaining traction)
- Capacitor plugin compatibility varies by platform (mitigated: minimal native API usage)

## References

- [SolidJS](https://www.solidjs.com/)
- [Kobalte](https://kobalte.dev/)
- [TanStack Table](https://tanstack.com/table)
- [Capacitor](https://capacitorjs.com/)
- [SolidJS vs React benchmark](https://krausest.github.io/js-framework-benchmark/)
