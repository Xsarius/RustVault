/**
 * Dialog component — modal overlay styled on Kobalte.
 */

import { type JSX, splitProps } from "solid-js";
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

  return (
    <KobalteDialog open={local.open} onOpenChange={local.onOpenChange}>
      <KobalteDialog.Portal>
        <KobalteDialog.Overlay class="fixed inset-0 z-[var(--z-overlay)] bg-black/50 data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[closed]:animate-out data-[closed]:fade-out-0" />
        <div class="fixed inset-0 z-[var(--z-modal)] flex items-center justify-center p-4">
          <KobalteDialog.Content class="w-full max-w-lg bg-bg border border-border rounded-[var(--radius-lg)] shadow-md data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 data-[closed]:animate-out data-[closed]:fade-out-0 data-[closed]:zoom-out-95">
            <div class="flex items-center justify-between p-6 pb-0">
              <div>
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
            <div class="p-6">{local.children}</div>
          </KobalteDialog.Content>
        </div>
      </KobalteDialog.Portal>
    </KobalteDialog>
  );
}
