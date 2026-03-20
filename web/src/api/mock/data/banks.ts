/**
 * Demo mode seed data — Banks.
 */

import type { Bank } from "~/api/types";

export const DEMO_BANKS: Bank[] = [
  {
    id: "bank-revolut",
    user_id: "demo-user",
    name: "Revolut",
    is_archived: false,
    sort_order: 0,
    metadata: {},
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "bank-lunar",
    user_id: "demo-user",
    name: "Lunar",
    is_archived: false,
    sort_order: 1,
    metadata: {},
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "bank-zen",
    user_id: "demo-user",
    name: "Zen",
    is_archived: false,
    sort_order: 2,
    metadata: {},
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];
