import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { TextField } from "~/components/ui";

const meta = {
  title: "UI/TextField",
  component: TextField,
  tags: ["autodocs"],
  argTypes: {
    type: {
      control: "select",
      options: ["text", "email", "password", "number", "url"],
    },
    required: { control: "boolean" },
    disabled: { control: "boolean" },
  },
} satisfies Meta<typeof TextField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    name: "username",
    label: "Username",
    placeholder: "Enter your username",
  },
};

export const Email: Story = {
  args: {
    name: "email",
    label: "Email Address",
    type: "email",
    placeholder: "you@example.com",
  },
};

export const Password: Story = {
  args: {
    name: "password",
    label: "Password",
    type: "password",
    placeholder: "••••••••",
  },
};

export const WithDescription: Story = {
  args: {
    name: "api-key",
    label: "API Key",
    description: "Find this in your account settings.",
    placeholder: "sk_live_...",
  },
};

export const WithError: Story = {
  args: {
    name: "email",
    label: "Email",
    type: "email",
    value: "not-an-email",
    error: "Please enter a valid email address.",
  },
};

export const Required: Story = {
  args: {
    name: "name",
    label: "Full Name",
    required: true,
    placeholder: "Jane Doe",
  },
};

export const Disabled: Story = {
  args: {
    name: "readonly",
    label: "Account ID",
    value: "acct_abc123",
    disabled: true,
  },
};

export const Number: Story = {
  args: {
    name: "amount",
    label: "Amount",
    type: "number",
    placeholder: "0.00",
  },
};

export const AllStates: Story = {
  render: () => (
    <div style={{ display: "flex", "flex-direction": "column", gap: "1rem", width: "320px" }}>
      <TextField name="normal" label="Normal" placeholder="Type here…" />
      <TextField name="desc" label="With Helper" description="Helper text below." />
      <TextField name="err" label="With Error" value="bad" error="This field is invalid." />
      <TextField name="req" label="Required" required placeholder="Required field" />
      <TextField name="dis" label="Disabled" value="Locked" disabled />
    </div>
  ),
};
