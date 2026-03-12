/**
 * Tooltip component — styled on Kobalte.
 */

import { type JSX } from "solid-js";
import { Tooltip as KobalteTooltip } from "@kobalte/core/tooltip";

export interface TooltipProps {
  /** Trigger element. */
  children: JSX.Element;
  /** Tooltip text. */
  content: string;
}

export function Tooltip(props: TooltipProps) {
  return (
    <KobalteTooltip>
      <KobalteTooltip.Trigger as="span" class="inline-flex">
        {props.children}
      </KobalteTooltip.Trigger>
      <KobalteTooltip.Portal>
        <KobalteTooltip.Content class="z-[var(--z-dropdown)] px-2.5 py-1.5 text-xs font-medium text-white bg-gray-900 dark:bg-gray-100 dark:text-gray-900 rounded-[var(--radius-sm)] shadow-sm data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 data-[closed]:animate-out data-[closed]:fade-out-0">
          {props.content}
          <KobalteTooltip.Arrow />
        </KobalteTooltip.Content>
      </KobalteTooltip.Portal>
    </KobalteTooltip>
  );
}
