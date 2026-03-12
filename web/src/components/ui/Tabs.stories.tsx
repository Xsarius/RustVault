import type { Meta, StoryObj } from "storybook-solidjs-vite";
import { createSignal } from "solid-js";
import { Tabs, TabList, TabTrigger, TabContent } from "~/components/ui";

const meta = {
  title: "UI/Tabs",
  component: Tabs,
  tags: ["autodocs"],
} satisfies Meta<typeof Tabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Uncontrolled: Story = {
  render: () => (
    <Tabs defaultValue="general" onChange={(v) => console.log("Tab:", v)}>
      <TabList>
        <TabTrigger value="general">General</TabTrigger>
        <TabTrigger value="security">Security</TabTrigger>
        <TabTrigger value="appearance">Appearance</TabTrigger>
      </TabList>
      <TabContent value="general">
        <p style={{ padding: "1rem", color: "var(--color-text)" }}>General settings content.</p>
      </TabContent>
      <TabContent value="security">
        <p style={{ padding: "1rem", color: "var(--color-text)" }}>Security settings content.</p>
      </TabContent>
      <TabContent value="appearance">
        <p style={{ padding: "1rem", color: "var(--color-text)" }}>Appearance settings content.</p>
      </TabContent>
    </Tabs>
  ),
};

export const Controlled: Story = {
  render: () => {
    const [tab, setTab] = createSignal("profile");
    return (
      <div>
        <p style={{ "margin-bottom": "0.5rem", color: "var(--color-text-secondary)", "font-size": "0.875rem" }}>
          Active tab: {tab()}
        </p>
        <Tabs value={tab()} onChange={setTab}>
          <TabList>
            <TabTrigger value="profile">Profile</TabTrigger>
            <TabTrigger value="billing">Billing</TabTrigger>
            <TabTrigger value="integrations">Integrations</TabTrigger>
          </TabList>
          <TabContent value="profile">
            <p style={{ padding: "1rem", color: "var(--color-text)" }}>Edit your profile details.</p>
          </TabContent>
          <TabContent value="billing">
            <p style={{ padding: "1rem", color: "var(--color-text)" }}>Manage your billing info.</p>
          </TabContent>
          <TabContent value="integrations">
            <p style={{ padding: "1rem", color: "var(--color-text)" }}>Connected services.</p>
          </TabContent>
        </Tabs>
      </div>
    );
  },
};

export const TwoTabs: Story = {
  render: () => (
    <Tabs defaultValue="income">
      <TabList>
        <TabTrigger value="income">Income</TabTrigger>
        <TabTrigger value="expenses">Expenses</TabTrigger>
      </TabList>
      <TabContent value="income">
        <p style={{ padding: "1rem", color: "var(--color-income)" }}>+€1,234.56</p>
      </TabContent>
      <TabContent value="expenses">
        <p style={{ padding: "1rem", color: "var(--color-expense)" }}>−€987.65</p>
      </TabContent>
    </Tabs>
  ),
};
