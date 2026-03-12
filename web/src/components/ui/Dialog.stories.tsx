import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { createSignal } from "solid-js";
import { Button, Dialog } from "~/components/ui";

const meta = {
  title: "UI/Dialog",
  component: Dialog,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
} satisfies Meta<typeof Dialog>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => {
    const [open, setOpen] = createSignal(false);
    return (
      <>
        <Button onClick={() => setOpen(true)}>Open Dialog</Button>
        <Dialog open={open()} onOpenChange={setOpen} title="Confirm Action">
          <p style={{ color: "var(--color-text-secondary)", "margin-bottom": "1rem" }}>
            Are you sure you want to proceed?
          </p>
          <div style={{ display: "flex", gap: "0.5rem", "justify-content": "flex-end" }}>
            <Button variant="secondary" onClick={() => setOpen(false)}>Cancel</Button>
            <Button onClick={() => setOpen(false)}>Confirm</Button>
          </div>
        </Dialog>
      </>
    );
  },
};

export const WithDescription: Story = {
  render: () => {
    const [open, setOpen] = createSignal(false);
    return (
      <>
        <Button variant="danger" onClick={() => setOpen(true)}>Delete Account</Button>
        <Dialog
          open={open()}
          onOpenChange={setOpen}
          title="Delete Account"
          description="This action cannot be undone."
        >
          <p style={{ color: "var(--color-text-secondary)", "margin-bottom": "1rem" }}>
            All your data will be permanently removed. Please type "DELETE" to confirm.
          </p>
          <div style={{ display: "flex", gap: "0.5rem", "justify-content": "flex-end" }}>
            <Button variant="secondary" onClick={() => setOpen(false)}>Cancel</Button>
            <Button variant="danger" onClick={() => setOpen(false)}>Delete</Button>
          </div>
        </Dialog>
      </>
    );
  },
};
