import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { createSignal } from "solid-js";
import { Checkbox } from "~/components/ui";

const meta = {
  title: "UI/Checkbox",
  component: Checkbox,
  tags: ["autodocs"],
  argTypes: {
    disabled: { control: "boolean" },
  },
} satisfies Meta<typeof Checkbox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => {
    const [checked, setChecked] = createSignal(false);
    return <Checkbox label="Accept terms" checked={checked()} onChange={setChecked} />;
  },
};

export const Checked: Story = {
  render: () => {
    const [checked, setChecked] = createSignal(true);
    return <Checkbox label="Show notifications" checked={checked()} onChange={setChecked} />;
  },
};

export const Disabled: Story = {
  render: () => (
    <Checkbox label="Read-only option" checked={false} onChange={() => {}} disabled />
  ),
};

export const DisabledChecked: Story = {
  render: () => (
    <Checkbox label="Locked setting" checked={true} onChange={() => {}} disabled />
  ),
};
