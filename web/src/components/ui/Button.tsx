/**
 * Button component — 4 variants styled on Kobalte.
 *
 * Variants: primary, secondary, ghost, danger.
 * Sizes: sm, md, lg.
 */

import { type JSX, splitProps } from "solid-js";
import { Button as KobalteButton } from "@kobalte/core/button";

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual variant. */
  variant?: "primary" | "secondary" | "ghost" | "danger";
  /** Size. */
  size?: "sm" | "md" | "lg";
  /** Show loading state. */
  loading?: boolean;
}

const variantClasses: Record<string, string> = {
  primary:
    "bg-primary text-white hover:bg-primary-hover focus-visible:ring-primary/50 disabled:opacity-50",
  secondary:
    "border border-border-strong bg-surface text-text hover:bg-surface-hover focus-visible:ring-primary/30 disabled:opacity-50",
  ghost:
    "text-text-secondary hover:bg-surface-hover hover:text-text focus-visible:ring-primary/30 disabled:opacity-50",
  danger:
    "bg-expense text-white hover:bg-red-700 focus-visible:ring-expense/50 disabled:opacity-50",
};

const sizeClasses: Record<string, string> = {
  sm: "h-8 px-3 text-xs gap-1.5 rounded-[var(--radius-sm)]",
  md: "h-9 px-4 text-sm gap-2 rounded-[var(--radius-md)]",
  lg: "h-10 px-5 text-sm gap-2 rounded-[var(--radius-md)]",
};

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, [
    "variant",
    "size",
    "loading",
    "class",
    "children",
    "disabled",
  ]);

  const variant = () => local.variant ?? "primary";
  const size = () => local.size ?? "md";

  return (
    <KobalteButton
      class={`inline-flex items-center justify-center font-medium transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 cursor-pointer select-none ${variantClasses[variant()]} ${sizeClasses[size()]} ${local.class ?? ""}`}
      disabled={local.disabled || local.loading}
      {...rest}
    >
      {local.loading && (
        <svg
          class="animate-spin h-4 w-4"
          viewBox="0 0 24 24"
          fill="none"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          />
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
          />
        </svg>
      )}
      {local.children}
    </KobalteButton>
  );
}
