import type { StorybookConfig } from "storybook-solidjs-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.@(ts|tsx)"],
  framework: "storybook-solidjs-vite",
  core: {
    disableTelemetry: true,
  },
  viteFinal(config) {
    config.resolve ??= {};
    config.resolve.alias = {
      ...config.resolve.alias,
      "~": "/src",
    };
    return config;
  },
};

export default config;
