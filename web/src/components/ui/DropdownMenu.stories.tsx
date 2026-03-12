import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { DropdownMenu, DropdownItem, DropdownSeparator, Button } from "~/components/ui";

const meta = {
  title: "UI/DropdownMenu",
  component: DropdownMenu,
  tags: ["autodocs"],
  parameters: {
    layout: "padded",
  },
} satisfies Meta<typeof DropdownMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <DropdownMenu trigger={<Button variant="secondary">Actions</Button>}>
      <DropdownItem onSelect={() => alert("Edit")}>Edit</DropdownItem>
      <DropdownItem onSelect={() => alert("Duplicate")}>Duplicate</DropdownItem>
      <DropdownSeparator />
      <DropdownItem onSelect={() => alert("Delete")} danger>Delete</DropdownItem>
    </DropdownMenu>
  ),
};

export const WithManyItems: Story = {
  render: () => (
    <DropdownMenu trigger={<Button>File</Button>}>
      <DropdownItem onSelect={() => {}}>New</DropdownItem>
      <DropdownItem onSelect={() => {}}>Open</DropdownItem>
      <DropdownItem onSelect={() => {}}>Save</DropdownItem>
      <DropdownItem onSelect={() => {}}>Save As…</DropdownItem>
      <DropdownSeparator />
      <DropdownItem onSelect={() => {}}>Export</DropdownItem>
      <DropdownItem onSelect={() => {}}>Print</DropdownItem>
      <DropdownSeparator />
      <DropdownItem onSelect={() => {}} danger>Close</DropdownItem>
    </DropdownMenu>
  ),
};
