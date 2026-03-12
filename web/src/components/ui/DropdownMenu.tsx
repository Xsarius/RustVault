/**
 * DropdownMenu — styled on Kobalte.
 */

import { type JSX, splitProps } from "solid-js";
import { DropdownMenu as KobalteDropdown } from "@kobalte/core/dropdown-menu";

export interface DropdownMenuProps {
  /** Trigger element. */
  trigger: JSX.Element;
  /** Menu content. */
  children: JSX.Element;
}

export function DropdownMenu(props: DropdownMenuProps) {
  return (
    <KobalteDropdown>
      <KobalteDropdown.Trigger class="outline-none" as="div">
        {props.trigger}
      </KobalteDropdown.Trigger>
      <KobalteDropdown.Portal>
        <KobalteDropdown.Content class="z-[var(--z-dropdown)] min-w-[180px] bg-bg border border-border rounded-[var(--radius-md)] shadow-sm p-1 data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 data-[closed]:animate-out data-[closed]:fade-out-0 data-[closed]:zoom-out-95 outline-none">
          {props.children}
        </KobalteDropdown.Content>
      </KobalteDropdown.Portal>
    </KobalteDropdown>
  );
}

export interface DropdownItemProps {
  /** Handler when clicked. */
  onSelect: () => void;
  /** Item content. */
  children: JSX.Element;
  /** Danger styling. */
  danger?: boolean;
}

export function DropdownItem(props: DropdownItemProps) {
  const [local] = splitProps(props, ["onSelect", "children", "danger"]);

  return (
    <KobalteDropdown.Item
      onSelect={local.onSelect}
      class={`flex items-center gap-2 px-3 py-2 text-sm rounded-[var(--radius-sm)] cursor-pointer outline-none data-[highlighted]:bg-surface-hover ${local.danger ? "text-expense data-[highlighted]:text-expense" : "text-text"}`}
    >
      {local.children}
    </KobalteDropdown.Item>
  );
}

export function DropdownSeparator() {
  return <KobalteDropdown.Separator class="h-px my-1 bg-border" />;
}
