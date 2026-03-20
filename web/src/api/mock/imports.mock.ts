/**
 * Demo mode — Imports mock API.
 *
 * In demo mode file imports are simulated. The upload parses nothing real;
 * it returns a canned preview so the import wizard UI can still be explored.
 */

import { simulate } from "./latency";
import type {
  Import,
  UploadResponse,
  ImportExecutionResult,
  ParsedRow,
  ApiResponse,
  PaginatedResponse,
} from "~/api/types";

const DEMO_IMPORTS: Import[] = [];

function makeImport(accountId: string, fileName: string): Import {
  return {
    id: `imp-${crypto.randomUUID()}`,
    user_id: "demo-user",
    file_name: fileName,
    file_format: "csv",
    account_id: accountId,
    status: "completed",
    total_rows: 5,
    imported_count: 5,
    skipped_count: 0,
    duplicate_count: 0,
    error_count: 0,
    error_details: null,
    column_mapping: null,
    metadata: {},
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

const DEMO_PREVIEW: ParsedRow[] = [
  { date: "2026-03-20", amount: "-12.50", currency: "EUR", description: "Demo row 1", payee: "Shop A", reference: null, metadata: {} },
  { date: "2026-03-21", amount: "-8.00",  currency: "EUR", description: "Demo row 2", payee: "Shop B", reference: null, metadata: {} },
  { date: "2026-03-22", amount: "-34.99", currency: "EUR", description: "Demo row 3", payee: "Shop C", reference: null, metadata: {} },
  { date: "2026-03-23", amount: "-5.50",  currency: "EUR", description: "Demo row 4", payee: "Shop D", reference: null, metadata: {} },
  { date: "2026-03-24", amount: "500.00", currency: "EUR", description: "Demo income", payee: null,    reference: null, metadata: {} },
];

export async function uploadFile(
  accountId: string,
  _formData: FormData,
): Promise<UploadResponse> {
  const imp = makeImport(accountId, "demo-statement.csv");
  DEMO_IMPORTS.push(imp);
  return simulate({
    import: imp,
    detected_format: "csv",
    preview: DEMO_PREVIEW,
    total_rows: DEMO_PREVIEW.length,
  });
}

export async function executeImport(
  importId: string,
  _body: unknown,
): Promise<ImportExecutionResult> {
  const imp = DEMO_IMPORTS.find((i) => i.id === importId) ?? makeImport("acc-revolut-eur", "demo.csv");
  return simulate({
    import: { ...imp, status: "completed" },
    imported_count: 5,
    duplicate_count: 0,
    error_count: 0,
    errors: [],
    rules_applied: {},
  });
}

export async function listImports(): Promise<PaginatedResponse<Import>> {
  return simulate({
    data: [...DEMO_IMPORTS],
    meta: { page_size: 50, has_more: false },
  });
}

export async function getImport(id: string): Promise<ApiResponse<Import>> {
  const imp = DEMO_IMPORTS.find((i) => i.id === id);
  if (!imp) throw new Error("Import not found");
  return simulate({ data: imp });
}

export async function rollbackImport(id: string): Promise<ApiResponse<Import>> {
  const imp = DEMO_IMPORTS.find((i) => i.id === id);
  if (!imp) throw new Error("Import not found");
  return simulate({ data: { ...imp, status: "rolled_back" } });
}
