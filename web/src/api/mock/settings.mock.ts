/**
 * Demo mode — Settings mock API.
 */

import { simulate } from "./latency";
import { demoStore, setDemoStore } from "./store";
import type { UserSettings, UpdateSettings, ApiResponse } from "~/api/types";

export async function getSettings(): Promise<ApiResponse<UserSettings>> {
  return simulate({ data: { ...demoStore.settings } });
}

export async function updateSettings(
  body: UpdateSettings,
): Promise<ApiResponse<UserSettings>> {
  setDemoStore("settings", (prev) => ({ ...prev, ...body }));
  return simulate({ data: { ...demoStore.settings } });
}
