import { create } from "zustand";
import type { ScanComplete, ScanItem, ScanProgress } from "../types";

interface SweepState {
  platform: string;
  scanning: boolean;
  progress: ScanProgress | null;
  items: ScanItem[];
  selected: Set<string>;
  complete: ScanComplete | null;
  dryRun: boolean;
  permanentDelete: boolean;
  lastError: string | null;

  setPlatform: (p: string) => void;
  setScanning: (v: boolean) => void;
  setProgress: (p: ScanProgress | null) => void;
  resetItems: () => void;
  addItem: (item: ScanItem) => void;
  setComplete: (c: ScanComplete | null) => void;
  toggleSelect: (id: string) => void;
  selectAllSafe: () => void;
  selectNone: () => void;
  selectPaths: (ids: string[]) => void;
  removeItemsByPaths: (paths: string[]) => void;
  setDryRun: (v: boolean) => void;
  setPermanentDelete: (v: boolean) => void;
  setLastError: (e: string | null) => void;
}

export const useSweepStore = create<SweepState>((set, get) => ({
  platform: "",
  scanning: false,
  progress: null,
  items: [],
  selected: new Set(),
  complete: null,
  dryRun: true,
  permanentDelete: false,
  lastError: null,

  setPlatform: (platform) => set({ platform }),
  setScanning: (scanning) => set({ scanning }),
  setProgress: (progress) => set({ progress }),
  resetItems: () => set({ items: [], selected: new Set(), complete: null }),
  addItem: (item) => set({ items: [...get().items, item] }),
  setComplete: (complete) => set({ complete, scanning: false }),
  toggleSelect: (id) => {
    const next = new Set(get().selected);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    set({ selected: next });
  },
  selectAllSafe: () => {
    const ids = get()
      .items.filter((i) => i.deletable && i.risk === "safe")
      .map((i) => i.id);
    set({ selected: new Set(ids) });
  },
  selectNone: () => set({ selected: new Set() }),
  selectPaths: (ids) => set({ selected: new Set(ids) }),
  removeItemsByPaths: (paths) => {
    const pathSet = new Set(paths);
    const items = get().items.filter((i) => !pathSet.has(i.path));
    const selected = new Set(
      [...get().selected].filter((id) => items.some((i) => i.id === id)),
    );
    set({ items, selected });
  },
  setDryRun: (dryRun) => set({ dryRun }),
  setPermanentDelete: (permanentDelete) => set({ permanentDelete }),
  setLastError: (lastError) => set({ lastError }),
}));
