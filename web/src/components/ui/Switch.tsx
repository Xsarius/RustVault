/**
 * Switch component — toggle switch styled on Kobalte.
 */

import { splitProps } from "solid-js";
import { Switch as KobalteSwitch } from "@kobalte/core/switch";

export interface SwitchProps {
  /** Label text. */
  label: string;
  /** Controlled checked state. */
  checked: boolean;
  /** Change handler. */
  onChange: (checked: boolean) => void;
  /** Whether disabled. */
  disabled?: boolean;
}

export function Switch(props: SwitchProps) {
  const [local] = splitProps(props, ["label", "checked", "onChange", "disabled"]);

  return (
    <KobalteSwitch
      class="flex items-center justify-between gap-3"
      checked={local.checked}
      onChange={local.onChange}
      disabled={local.disabled}
    >
      <KobalteSwitch.Label class="text-sm text-text select-none cursor-pointer">
        {local.label}
      </KobalteSwitch.Label>
      <KobalteSwitch.Input />
      <KobalteSwitch.Control class="inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border border-border-strong bg-surface-hover transition-colors data-[checked]:bg-primary data-[checked]:border-primary disabled:cursor-not-allowed disabled:opacity-50">
        <KobalteSwitch.Thumb class="block h-4 w-4 rounded-full bg-white shadow-sm transition-transform data-[checked]:translate-x-4 translate-x-0.5" />
      </KobalteSwitch.Control>
    </KobalteSwitch>
  );
}
