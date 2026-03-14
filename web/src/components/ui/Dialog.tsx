/**
 * Dialog component — modal overlay styled on Kobalte.
 */

import { type JSX, splitProps, createEffect } from "solid-js";
import { Dialog as KobalteDialog } from "@kobalte/core/dialog";
import { X } from "lucide-solid";

export interface DialogProps {
  /** Controlled open state. */
  open: boolean;
  /** Called when dialog should close. */
  onOpenChange: (open: boolean) => void;
  /** Dialog title (required for a11y). */
  title: string;
  /** Optional description below title. */
  description?: string;
  /** Dialog content. */
  children: JSX.Element;
}

export function Dialog(props: DialogProps) {
  const [local] = splitProps(props, [
    "open",
    "onOpenChange",
    "title",
    "description",
    "children",
  ]);

  createEffect(() => {
    if (!local.open) return;
    const active = document.activeElement;
    if (active instanceof HTMLElement) {
      active.blur();
    }
  });

  return (
    <KobalteDialog open={local.open} onOpenChange={local.onOpenChange}>
      <KobalteDialog.Portal>
        {/* Overlay handles backdrop click-to-close. */}
        <KobalteDialog.Overlay
          class="fixed inset-0 z-[var(--z-overlay)] bg-black/50 data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[closed]:animate-out data-[closed]:fade-out-0"
          onClick={() => local.onOpenChange(false)}
        />
        {/* pointer-events-none so backdrop clicks reach the Overlay above. */}
        <div class="fixed inset-0 z-[var(--z-modal)] flex items-center justify-center p-4 pointer-events-none">
          <KobalteDialog.Content
            class="w-full max-w-lg bg-bg border border-border rounded-[var(--radius-lg)] shadow-md overflow-hidden data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 data-[closed]:animate-out data-[closed]:fade-out-0 data-[closed]:zoom-out-95 pointer-events-auto"
            onInteractOutside={(e) => e.preventDefault()}
          >
            <div class="flex items-center justify-between p-6 pb-0">
              <div class="min-w-0 flex-1 pr-2">
                <KobalteDialog.Title class="text-lg font-semibold text-text">
                  {local.title}
                </KobalteDialog.Title>
                {local.description && (
                  <KobalteDialog.Description class="mt-1 text-sm text-text-secondary">
                    {local.description}
                  </KobalteDialog.Description>
                )}
              </div>
              <KobalteDialog.CloseButton class="p-1 rounded-[var(--radius-sm)] text-text-tertiary hover:text-text hover:bg-surface-hover transition-colors cursor-pointer">
                <X size={18} />
              </KobalteDialog.CloseButton>
            </div>
            <div class="p-6 overflow-y-auto max-h-[calc(90vh-5rem)]">{local.children}</div>
          </KobalteDialog.Content>
        </div>
      </KobalteDialog.Portal>
    </KobalteDialog>
  );
}
