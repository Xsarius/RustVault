/**
 * Checkbox component — styled on Kobalte.
 */

import { splitProps } from "solid-js";
import { Checkbox as KobalteCheckbox } from "@kobalte/core/checkbox";
import { Check } from "lucide-solid";

export interface CheckboxProps {
  /** Label text. */
  label: string;
  /** Controlled checked state. */
  checked: boolean;
  /** Change handler. */
  onChange: (checked: boolean) => void;
  /** Whether disabled. */
  disabled?: boolean;
}

export function Checkbox(props: CheckboxProps) {
  const [local] = splitProps(props, ["label", "checked", "onChange", "disabled"]);

  return (
    <KobalteCheckbox
      class="flex items-center gap-2"
      checked={local.checked}
      onChange={local.onChange}
      disabled={local.disabled}
    >
      <KobalteCheckbox.Input />
      <KobalteCheckbox.Control class="flex items-center justify-center h-4 w-4 rounded-[var(--radius-sm)] border border-border-strong bg-bg transition-colors data-[checked]:bg-primary data-[checked]:border-primary cursor-pointer">
        <KobalteCheckbox.Indicator>
          <Check size={12} class="text-white" />
        </KobalteCheckbox.Indicator>
      </KobalteCheckbox.Control>
      <KobalteCheckbox.Label class="text-sm text-text select-none cursor-pointer">
        {local.label}
      </KobalteCheckbox.Label>
    </KobalteCheckbox>
  );
}
