import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { Skeleton, ListSkeleton, DashboardSkeleton } from "~/components/ui";

const meta = {
  title: "UI/Skeleton",
  component: Skeleton,
  tags: ["autodocs"],
  argTypes: {
    variant: {
      control: "select",
      options: ["line", "circle", "rect"],
    },
  },
} satisfies Meta<typeof Skeleton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Line: Story = {
  args: {
    variant: "line",
  },
  decorators: [
    (Story: () => import("solid-js").JSX.Element) => (
      <div style={{ width: "300px" }}>
        <Story />
      </div>
    ),
  ],
};

export const Circle: Story = {
  args: {
    variant: "circle",
    class: "w-12 h-12",
  },
};

export const Rectangle: Story = {
  args: {
    variant: "rect",
    class: "w-64 h-32",
  },
};

export const ListPlaceholder: Story = {
  render: () => (
    <div style={{ width: "400px" }}>
      <ListSkeleton />
    </div>
  ),
};

export const DashboardPlaceholder: Story = {
  render: () => (
    <div style={{ width: "800px" }}>
      <DashboardSkeleton />
    </div>
  ),
};

export const CustomComposition: Story = {
  render: () => (
    <div style={{ display: "flex", gap: "0.75rem", "align-items": "center", width: "300px" }}>
      <Skeleton variant="circle" class="w-10 h-10 shrink-0" />
      <div style={{ flex: "1", display: "flex", "flex-direction": "column", gap: "0.5rem" }}>
        <Skeleton variant="line" class="w-3/4" />
        <Skeleton variant="line" class="w-1/2" />
      </div>
    </div>
  ),
};
