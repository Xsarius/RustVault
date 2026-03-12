/**
 * Skeleton loading placeholder.
 *
 * Variants for different content shapes used in page-level skeletons.
 */

import { splitProps } from "solid-js";

export interface SkeletonProps {
  /** Width — Tailwind class or inline style. */
  class?: string;
  /** Visual shape. */
  variant?: "line" | "circle" | "rect";
}

export function Skeleton(props: SkeletonProps) {
  const [local] = splitProps(props, ["class", "variant"]);
  const variant = () => local.variant ?? "line";

  const shapeClass = () => {
    switch (variant()) {
      case "circle":
        return "rounded-full aspect-square";
      case "rect":
        return "rounded-[var(--radius-md)]";
      default:
        return "rounded-[var(--radius-sm)] h-4";
    }
  };

  return (
    <div
      class={`animate-pulse bg-surface-hover ${shapeClass()} ${local.class ?? ""}`}
    />
  );
}

/** Page skeleton for list views. */
export function ListSkeleton() {
  return (
    <div class="space-y-3 p-6">
      <Skeleton class="h-8 w-48" />
      <div class="space-y-2">
        {Array.from({ length: 8 }).map(() => (
          <Skeleton class="h-12 w-full" variant="rect" />
        ))}
      </div>
    </div>
  );
}

/** Page skeleton for dashboard / grid. */
export function DashboardSkeleton() {
  return (
    <div class="p-6 space-y-6">
      <Skeleton class="h-8 w-64" />
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Skeleton class="h-28" variant="rect" />
        <Skeleton class="h-28" variant="rect" />
        <Skeleton class="h-28" variant="rect" />
      </div>
      <Skeleton class="h-64" variant="rect" />
    </div>
  );
}
