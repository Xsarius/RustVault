/**
 * Toast notification system — non-blocking, accessible.
 *
 * Uses Kobalte Toast with auto-dismiss.
 */

import { Toast as KobalteToast, toaster } from "@kobalte/core/toast";
import { X } from "lucide-solid";

export interface ToastData {
  title: string;
  description?: string;
  variant?: "success" | "error" | "warning" | "info";
}

const variantStyles: Record<string, string> = {
  success: "border-income/30 bg-bg",
  error: "border-expense/30 bg-bg",
  warning: "border-warning/30 bg-bg",
  info: "border-primary/30 bg-bg",
};

const dotStyles: Record<string, string> = {
  success: "bg-income",
  error: "bg-expense",
  warning: "bg-warning",
  info: "bg-primary",
};

/** Show a toast notification. */
export function showToast(data: ToastData) {
  toaster.show((props) => (
    <KobalteToast
      toastId={props.toastId}
      class={`flex items-start gap-3 p-4 rounded-[var(--radius-md)] border shadow-sm ${variantStyles[data.variant ?? "info"]} data-[opened]:animate-in data-[opened]:slide-in-from-right-full data-[closed]:animate-out data-[closed]:fade-out-80 data-[closed]:slide-out-to-right-full data-[swipe=move]:translate-x-[var(--kb-toast-swipe-move-x)] data-[swipe=cancel]:translate-x-0 data-[swipe=end]:animate-out data-[swipe=end]:slide-out-to-right-full`}
    >
      <div
        class={`mt-1 h-2 w-2 shrink-0 rounded-full ${dotStyles[data.variant ?? "info"]}`}
      />
      <div class="flex-1 min-w-0">
        <KobalteToast.Title class="text-sm font-medium text-text">
          {data.title}
        </KobalteToast.Title>
        {data.description && (
          <KobalteToast.Description class="mt-1 text-xs text-text-secondary">
            {data.description}
          </KobalteToast.Description>
        )}
      </div>
      <KobalteToast.CloseButton class="p-0.5 rounded-[var(--radius-sm)] text-text-tertiary hover:text-text transition-colors cursor-pointer">
        <X size={14} />
      </KobalteToast.CloseButton>
    </KobalteToast>
  ));
}

/** Toast region component — place once at the app root. */
export function ToastRegion() {
  return (
    <KobalteToast.Region duration={4000}>
      <KobalteToast.List class="fixed bottom-4 right-4 z-[var(--z-toast)] flex flex-col gap-2 w-80 outline-none" />
    </KobalteToast.Region>
  );
}
