/**
 * Select component — dropdown styled on Kobalte.
 */

import { splitProps } from "solid-js";
import { Select as KobalteSelect } from "@kobalte/core/select";
import { ChevronDown, Check } from "lucide-solid";

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps {
  /** Select name. */
  name: string;
  /** Label text. */
  label: string;
  /** Available options. */
  options: SelectOption[];
  /** Current value. */
  value?: string;
  /** Change handler. */
  onChange?: (value: string) => void;
  /** Placeholder when no value selected. */
  placeholder?: string;
  /** Whether the field is required. */
  required?: boolean;
  /** Whether the field is disabled. */
  disabled?: boolean;
}

export function Select(props: SelectProps) {
  const [local] = splitProps(props, [
    "name",
    "label",
    "options",
    "value",
    "onChange",
    "placeholder",
    "required",
    "disabled",
  ]);

  const selectedOption = () => local.options.find((o) => o.value === local.value);

  return (
    <KobalteSelect<SelectOption>
      options={local.options}
      optionValue="value"
      optionTextValue="label"
      value={selectedOption()}
      onChange={(option) => {
        if (option && local.onChange) local.onChange(option.value);
      }}
      placeholder={local.placeholder ?? "Select…"}
      disallowEmptySelection={local.required}
      disabled={local.disabled}
      itemComponent={(itemProps) => (
        <KobalteSelect.Item
          item={itemProps.item}
          class="flex items-center justify-between px-3 py-2 text-sm text-text rounded-[var(--radius-sm)] cursor-pointer outline-none data-[highlighted]:bg-surface-hover data-[highlighted]:text-text"
        >
          <KobalteSelect.ItemLabel>{itemProps.item.rawValue.label}</KobalteSelect.ItemLabel>
          <KobalteSelect.ItemIndicator>
            <Check size={14} class="text-primary" />
          </KobalteSelect.ItemIndicator>
        </KobalteSelect.Item>
      )}
    >
      <div class="flex flex-col gap-1.5">
        <KobalteSelect.Label class="text-[13px] font-medium text-text-secondary">
          {local.label}
          {local.required && <span class="text-expense ml-0.5">*</span>}
        </KobalteSelect.Label>
        <KobalteSelect.Trigger class="flex items-center justify-between h-9 w-full rounded-[var(--radius-md)] border border-border-strong bg-bg px-3 text-sm text-text focus:outline-none focus:ring-2 focus:ring-primary/40 focus:border-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer">
          <KobalteSelect.Value<SelectOption>>
            {(state) => state.selectedOption()?.label ?? local.placeholder ?? "Select…"}
          </KobalteSelect.Value>
          <KobalteSelect.Icon>
            <ChevronDown size={16} class="text-text-tertiary" />
          </KobalteSelect.Icon>
        </KobalteSelect.Trigger>
      </div>
      <KobalteSelect.Portal>
        <KobalteSelect.Content class="z-[var(--z-dropdown)] bg-bg border border-border rounded-[var(--radius-md)] shadow-sm overflow-hidden data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 data-[closed]:animate-out data-[closed]:fade-out-0 data-[closed]:zoom-out-95">
          <KobalteSelect.Listbox class="p-1 max-h-60 overflow-y-auto outline-none" />
        </KobalteSelect.Content>
      </KobalteSelect.Portal>
    </KobalteSelect>
  );
}
