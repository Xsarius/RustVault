/**
 * TextField / input component — form field with label, error, and description.
 */

import { type JSX, splitProps, Show } from "solid-js";
import { TextField as KobalteTextField } from "@kobalte/core/text-field";

export interface TextFieldProps {
  /** Input name. */
  name: string;
  /** Label text. */
  label: string;
  /** Input type. */
  type?: "text" | "email" | "password" | "number" | "url";
  /** Placeholder. */
  placeholder?: string;
  /** Current value. */
  value?: string;
  /** Change handler. */
  onInput?: JSX.EventHandler<HTMLInputElement, InputEvent>;
  /** Error message. */
  error?: string;
  /** Description/helper text. */
  description?: string;
  /** Whether the field is required. */
  required?: boolean;
  /** Whether the field is disabled. */
  disabled?: boolean;
  /** Auto-focus on mount. */
  autofocus?: boolean;
}

export function TextField(props: TextFieldProps) {
  const [local] = splitProps(props, [
    "name",
    "label",
    "type",
    "placeholder",
    "value",
    "onInput",
    "error",
    "description",
    "required",
    "disabled",
    "autofocus",
  ]);

  return (
    <KobalteTextField
      class="flex flex-col gap-1.5"
      name={local.name}
      value={local.value}
      validationState={local.error ? "invalid" : "valid"}
      required={local.required}
      disabled={local.disabled}
    >
      <KobalteTextField.Label class="text-[13px] font-medium text-text-secondary">
        {local.label}
        {local.required && <span class="text-expense ml-0.5">*</span>}
      </KobalteTextField.Label>
      <KobalteTextField.Input
        type={local.type ?? "text"}
        placeholder={local.placeholder}
        onInput={local.onInput}
        autofocus={local.autofocus}
        class="h-9 w-full rounded-[var(--radius-md)] border border-border-strong bg-bg px-3 text-sm text-text placeholder:text-text-tertiary focus:outline-none focus:ring-2 focus:ring-primary/40 focus:border-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      />
      <Show when={local.description && !local.error}>
        <KobalteTextField.Description class="text-xs text-text-tertiary">
          {local.description}
        </KobalteTextField.Description>
      </Show>
      <Show when={local.error}>
        <KobalteTextField.ErrorMessage class="text-xs text-expense">
          {local.error}
        </KobalteTextField.ErrorMessage>
      </Show>
    </KobalteTextField>
  );
}
