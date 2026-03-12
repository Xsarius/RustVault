/**
 * Tabs component — styled on Kobalte.
 */

import { type JSX, splitProps } from "solid-js";
import { Tabs as KobalteTabs } from "@kobalte/core/tabs";

export interface TabsProps {
  /** The currently active tab value (controlled). */
  value?: string;
  /** Default tab value (uncontrolled). */
  defaultValue?: string;
  /** Change handler. */
  onChange?: (value: string) => void;
  /** Tab items. */
  children: JSX.Element;
}

export function Tabs(props: TabsProps) {
  const [local] = splitProps(props, ["value", "defaultValue", "onChange", "children"]);

  return (
    <KobalteTabs
      value={local.value}
      defaultValue={local.defaultValue}
      onChange={local.onChange}
      class="flex flex-col"
    >
      {local.children}
    </KobalteTabs>
  );
}

export interface TabListProps {
  children: JSX.Element;
}

export function TabList(props: TabListProps) {
  return (
    <KobalteTabs.List class="flex border-b border-border">
      {props.children}
    </KobalteTabs.List>
  );
}

export interface TabTriggerProps {
  value: string;
  children: JSX.Element;
}

export function TabTrigger(props: TabTriggerProps) {
  return (
    <KobalteTabs.Trigger
      value={props.value}
      class="px-4 py-2 text-sm font-medium text-text-secondary border-b-2 border-transparent cursor-pointer transition-colors hover:text-text data-[selected]:text-primary data-[selected]:border-primary outline-none focus-visible:ring-2 focus-visible:ring-primary/30 rounded-t-[var(--radius-sm)]"
    >
      {props.children}
    </KobalteTabs.Trigger>
  );
}

export interface TabContentProps {
  value: string;
  children: JSX.Element;
}

export function TabContent(props: TabContentProps) {
  return (
    <KobalteTabs.Content value={props.value} class="pt-4 outline-none">
      {props.children}
    </KobalteTabs.Content>
  );
}
