import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { createSignal } from "solid-js";
import { Select } from "~/components/ui";

const currencies = [
  { value: "usd", label: "US Dollar" },
  { value: "eur", label: "Euro" },
  { value: "gbp", label: "British Pound" },
  { value: "jpy", label: "Japanese Yen" },
  { value: "pln", label: "Polish Złoty" },
];

const meta = {
  title: "UI/Select",
  component: Select,
  tags: ["autodocs"],
  argTypes: {
    required: { control: "boolean" },
    disabled: { control: "boolean" },
  },
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => {
    const [value, setValue] = createSignal("");
    return (
      <Select
        name="currency"
        label="Currency"
        options={currencies}
        value={value()}
        onChange={setValue}
      />
    );
  },
};

export const WithPlaceholder: Story = {
  render: () => {
    const [value, setValue] = createSignal("");
    return (
      <Select
        name="currency"
        label="Currency"
        options={currencies}
        value={value()}
        onChange={setValue}
        placeholder="Choose a currency…"
      />
    );
  },
};

export const Required: Story = {
  render: () => {
    const [value, setValue] = createSignal("");
    return (
      <Select
        name="currency"
        label="Currency"
        options={currencies}
        value={value()}
        onChange={setValue}
        required
      />
    );
  },
};

export const Disabled: Story = {
  render: () => (
    <Select
      name="currency"
      label="Currency"
      options={currencies}
      value="eur"
      onChange={() => {}}
      disabled
    />
  ),
};

export const Preselected: Story = {
  render: () => {
    const [value, setValue] = createSignal("pln");
    return (
      <Select
        name="currency"
        label="Currency"
        options={currencies}
        value={value()}
        onChange={setValue}
      />
    );
  },
};
