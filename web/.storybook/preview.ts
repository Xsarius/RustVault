import type { Preview } from "storybook-solidjs-vite";
import "../src/app.css";

const preview: Preview = {
  parameters: {
    backgrounds: {
      values: [
        { name: "light", value: "#ffffff" },
        { name: "dark", value: "#09090b" },
      ],
    },
    layout: "centered",
  },
  decorators: [
    (Story, context) => {
      const isDark =
        context.globals?.backgrounds?.value === "#09090b" ||
        context.parameters?.backgrounds?.default === "dark";

      if (isDark) {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }

      return Story();
    },
  ],
};

export default preview;
