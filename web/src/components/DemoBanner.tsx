/**
 * DemoBanner — shown in demo mode to inform visitors that they are
 * exploring sample data and that changes are not persisted.
 */

import { createSignal, Show, type Component } from "solid-js";

const GITHUB_URL = "https://github.com/kubajaloszynski/RustVault";

const DemoBanner: Component = () => {
  const [dismissed, setDismissed] = createSignal(false);

  return (
    <Show when={!dismissed()}>
      <div
        role="banner"
        style={{
          background: "rgba(217, 119, 6, 0.12)",
          "border-bottom": "1px solid rgba(217, 119, 6, 0.4)",
          color: "#92400e",
          padding: "0.55rem 1rem",
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          gap: "0.75rem",
          "font-size": "0.875rem",
          "line-height": "1.4",
        }}
      >
        <span>
          <strong>Demo mode</strong> — This is a live demo with sample data.{" "}
          <strong>Changes are not saved.</strong>{" "}
          <a
            href={GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
            style={{ color: "#b45309", "text-decoration": "underline" }}
          >
            View source on GitHub ↗
          </a>
        </span>
        <button
          type="button"
          aria-label="Dismiss demo banner"
          onClick={() => setDismissed(true)}
          style={{
            background: "none",
            border: "none",
            cursor: "pointer",
            color: "#92400e",
            "font-size": "1.1rem",
            "line-height": "1",
            padding: "0 0.25rem",
            "flex-shrink": "0",
          }}
        >
          ×
        </button>
      </div>
    </Show>
  );
};

export default DemoBanner;
