import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { Button, showToast, ToastRegion } from "~/components/ui";

const meta = {
  title: "UI/Toast",
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
  decorators: [
    (Story) => (
      <>
        <Story />
        <ToastRegion />
      </>
    ),
  ],
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const Success: Story = {
  render: () => (
    <Button
      onClick={() =>
        showToast({ title: "Transaction saved", description: "Your changes have been saved.", variant: "success" })
      }
    >
      Show Success Toast
    </Button>
  ),
};

export const Error: Story = {
  render: () => (
    <Button
      variant="danger"
      onClick={() =>
        showToast({ title: "Import failed", description: "The CSV file could not be parsed.", variant: "error" })
      }
    >
      Show Error Toast
    </Button>
  ),
};

export const Warning: Story = {
  render: () => (
    <Button
      variant="secondary"
      onClick={() =>
        showToast({ title: "Budget exceeded", description: "You're over budget by €45.00.", variant: "warning" })
      }
    >
      Show Warning Toast
    </Button>
  ),
};

export const Info: Story = {
  render: () => (
    <Button
      variant="ghost"
      onClick={() =>
        showToast({ title: "Sync complete", description: "All accounts are up to date.", variant: "info" })
      }
    >
      Show Info Toast
    </Button>
  ),
};

export const TitleOnly: Story = {
  render: () => (
    <Button onClick={() => showToast({ title: "Copied to clipboard" })}>
      Show Minimal Toast
    </Button>
  ),
};

export const AllVariants: Story = {
  render: () => (
    <div style={{ display: "flex", gap: "0.5rem", "flex-wrap": "wrap" }}>
      <Button onClick={() => showToast({ title: "Success", variant: "success" })}>Success</Button>
      <Button onClick={() => showToast({ title: "Error", variant: "error" })}>Error</Button>
      <Button onClick={() => showToast({ title: "Warning", variant: "warning" })}>Warning</Button>
      <Button onClick={() => showToast({ title: "Info", variant: "info" })}>Info</Button>
    </div>
  ),
};
