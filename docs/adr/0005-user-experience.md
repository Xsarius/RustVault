# ADR-0005: User Experience

- **Status:** Accepted
- **Date:** 2026-03-03
- **Deciders:** RustVault core team
- **Tags:** ux, i18n, accessibility, performance, design

## Context

RustVault is a self-hosted finance app used by individuals and households worldwide. The UX must:

- Feel instant — interactions respond in <100ms, pages load in <1s
- Work across languages, currencies, and date formats from day one
- Be accessible to users with disabilities (WCAG 2.1 AA)
- Scale from mobile phones to wide desktop monitors
- Handle large datasets (10k+ transactions) without degradation

## Decisions

### Internationalization: Fluent (backend) + ICU MessageFormat (frontend)

Two i18n systems, each optimized for its environment:

- **Backend:** Project Fluent (`fluent-rs`) for API error messages and server-generated content. Fluent handles plurals, gender, and grammatical cases natively.
- **Frontend:** ICU MessageFormat via `@solid-primitives/i18n` for reactive UI labels. Lazy-loaded — only the active locale is fetched.
- **Formatting:** ICU4X (Rust) and browser Intl API for locale-aware dates, numbers, and currencies.

| Alternative | Why not |
|-------------|---------|
| **gettext** | Basic plural support; no gender/case handling |
| **i18next** | JS-only; backend needs a separate solution |
| **Single system everywhere** | No mature Rust ICU implementation; Fluent is more expressive |

Fallback chain: `user.locale → browser locale → instance default → en-US`.

### Accessibility

- Kobalte headless components provide ARIA attributes, keyboard navigation, and focus management out of the box
- Semantic HTML throughout — no `<div>` buttons
- Color contrast ratios meet WCAG 2.1 AA
- All interactive elements are keyboard-navigable
- Screen reader announcements for dynamic content (import progress, form validation)

### Performance Targets

| Metric | Target |
|--------|--------|
| First Contentful Paint | < 1.0s |
| Time to Interactive | < 1.5s |
| Interaction response | < 100ms |
| Table scroll (10k rows) | 60fps |
| JS bundle (initial) | < 150KB gzipped |

Achieved via SolidJS fine-grained reactivity, virtualized lists, code splitting, lazy-loaded charts, and Brotli compression.

### Responsive Design

- Desktop-first layout with sidebar navigation
- Collapsible sidebar for narrow viewports
- Bottom navigation on mobile (≤768px)
- Touch-friendly tap targets (≥44px)
- Adaptive data tables — column hiding on small screens, horizontal scroll for dense data

### Design System

- **Tailwind CSS** for utility-first styling — no runtime CSS-in-JS overhead
- **Inter** for UI text — optimized for screen readability
- **JetBrains Mono** for monetary values — monospace digits align in columns
- Consistent spacing scale (4px base grid)
- Dark and light theme support via CSS custom properties

### Typography for Finance

Monetary amounts use tabular (monospace) numerals so columns of numbers align vertically. Currency symbols are placed according to locale rules (€100 vs 100€ vs $100).

## Consequences

### Positive

- Full i18n from day one — no retrofitting pain
- Fluent gives translators control over sentence structure without developer intervention
- Kobalte's headless components guarantee accessibility baseline
- Performance budget enforced via Lighthouse CI gates
- Responsive layout works for phone → ultrawide without separate codebases

### Negative

- Two i18n systems to maintain (Fluent + ICU) — key naming conventions must stay synchronized
- Fluent is less known than gettext — contributor learning curve for translators
- Strict performance budget may constrain feature complexity

### Risks

- Translation sync between `.ftl` and `.json` needs CI validation (mitigated: missing-key CI checks)
- Accessibility compliance requires ongoing testing, not just initial implementation (mitigated: axe-core in CI)

## References

- [Project Fluent](https://projectfluent.org/)
- [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/)
- [WCAG 2.1 AA](https://www.w3.org/WAI/WCAG21/quickref/)
- [Kobalte accessibility](https://kobalte.dev/docs/core/overview/accessibility)
- [Web performance budgets](https://web.dev/performance-budgets-101/)
