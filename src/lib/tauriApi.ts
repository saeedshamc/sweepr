import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AuditEntry,
  DeleteResult,
  ScanComplete,
  ScanItem,
  ScanProgress,
} from "../types";
import { useSweepStore } from "../store/sweepStore";

let unlisteners: UnlistenFn[] = [];

export async function initPlatform(): Promise<void> {
  const platform = await invoke<string>("get_platform");
  useSweepStore.getState().setPlatform(platform);
}

export async function attachScanListeners(): Promise<void> {
  await detachScanListeners();

  const store = useSweepStore.getState();

  unlisteners.push(
    await listen<ScanItem>("scan-item", (event) => {
      useSweepStore.getState().addItem(event.payload);
    }),
  );

  unlisteners.push(
    await listen<ScanProgress>("scan-progress", (event) => {
      useSweepStore.getState().setProgress(event.payload);
    }),
  );

  unlisteners.push(
    await listen<ScanComplete>("scan-complete", (event) => {
      useSweepStore.getState().setComplete(event.payload);
      useSweepStore.getState().setScanning(false);
    }),
  );

  void store;
}

export async function detachScanListeners(): Promise<void> {
  for (const u of unlisteners) {
    u();
  }
  unlisteners = [];
}

export async function startScan(): Promise<void> {
  const store = useSweepStore.getState();
  store.setLastError(null);
  store.resetItems();
  store.setScanning(true);
  store.setProgress(null);
  try {
    await invoke("start_scan");
  } catch (e) {
    store.setScanning(false);
    store.setLastError(String(e));
  }
}

export async function deleteSelected(
  paths: string[],
  dryRun: boolean,
  permanent: boolean,
): Promise<DeleteResult> {
  return invoke<DeleteResult>("delete_selected", {
    req: { paths, permanent, dry_run: dryRun },
  });
}

export async function fetchAuditLog(limit = 50): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("get_audit_log", { limit });
}
