import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { createSignal } from "solid-js";
import { Switch } from "~/components/ui";

const meta = {
  title: "UI/Switch",
  component: Switch,
  tags: ["autodocs"],
  argTypes: {
    disabled: { control: "boolean" },
  },
} satisfies Meta<typeof Switch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => {
    const [checked, setChecked] = createSignal(false);
    return <Switch label="Enable notifications" checked={checked()} onChange={setChecked} />;
  },
};

export const On: Story = {
  render: () => {
    const [checked, setChecked] = createSignal(true);
    return <Switch label="Dark mode" checked={checked()} onChange={setChecked} />;
  },
};

export const Disabled: Story = {
  render: () => (
    <Switch label="Premium feature" checked={false} onChange={() => {}} disabled />
  ),
};

export const DisabledOn: Story = {
  render: () => (
    <Switch label="Always active" checked={true} onChange={() => {}} disabled />
  ),
};

export const Group: Story = {
  render: () => {
    const [a, setA] = createSignal(true);
    const [b, setB] = createSignal(false);
    const [c, setC] = createSignal(true);
    return (
      <div style={{ display: "flex", "flex-direction": "column", gap: "1rem", width: "280px" }}>
        <Switch label="Email notifications" checked={a()} onChange={setA} />
        <Switch label="Push notifications" checked={b()} onChange={setB} />
        <Switch label="SMS notifications" checked={c()} onChange={setC} />
      </div>
    );
  },
};
