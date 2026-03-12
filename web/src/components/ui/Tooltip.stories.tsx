import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { Button, Tooltip } from "~/components/ui";

const meta = {
  title: "UI/Tooltip",
  component: Tooltip,
  tags: ["autodocs"],
} satisfies Meta<typeof Tooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <Tooltip content="Save your changes">
      <Button>Hover me</Button>
    </Tooltip>
  ),
};

export const OnIconButton: Story = {
  render: () => (
    <Tooltip content="Settings">
      <Button variant="ghost" size="sm">
        ⚙️
      </Button>
    </Tooltip>
  ),
};

export const LongContent: Story = {
  render: () => (
    <Tooltip content="This tooltip has longer content that explains the feature in more detail.">
      <Button variant="secondary">Help</Button>
    </Tooltip>
  ),
};
