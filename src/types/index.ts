export type RiskTier = "safe" | "caution" | "risky";

export interface ScanItem {
  id: string;
  rule_id: string;
  category: string;
  name: string;
  path: string;
  size_bytes: number;
  file_count: number;
  last_modified: string | null;
  risk: RiskTier;
  deletable: boolean;
  informational_only: boolean;
  cleanup_hint: string | null;
}

export interface ScanProgress {
  current_path: string;
  rule_id: string;
  items_found: number;
  bytes_found: number;
}

export interface ScanComplete {
  item_count: number;
  total_bytes: number;
  duration_ms: number;
  errors: string[];
}

export interface DeleteItemResult {
  path: string;
  success: boolean;
  dry_run: boolean;
  error: string | null;
  size_bytes: number;
}

export interface DeleteResult {
  results: DeleteItemResult[];
  freed_bytes: number;
  failed: number;
}

export interface AuditEntry {
  timestamp: string;
  path: string;
  size_bytes: number;
  permanent: boolean;
  dry_run: boolean;
  success: boolean;
  error: string | null;
}

export const CATEGORY_LABELS: Record<string, string> = {
  temp: "Temporary files",
  app_caches: "App caches",
  package_managers: "Package managers",
  build_artifacts: "Build artifacts",
  downloads: "Downloads",
  logs: "Logs",
  docker: "Docker",
  recycle_bin: "Recycle Bin",
  updates: "Updates",
  system: "System",
};
