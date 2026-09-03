import { useEffect, useMemo, useState } from "react";
import { CategoryChart } from "./components/CategoryChart";
import { ConfirmDialog } from "./components/ConfirmDialog";
import { ResultsList } from "./components/ResultsList";
import { formatBytes } from "./lib/format";
import {
  attachScanListeners,
  deleteSelected,
  detachScanListeners,
  initPlatform,
  startScan,
} from "./lib/tauriApi";
import { useSweepStore } from "./store/sweepStore";
import "./App.css";

function App() {
  const {
    platform,
    scanning,
    progress,
    items,
    selected,
    complete,
    dryRun,
    permanentDelete,
    lastError,
    toggleSelect,
    selectAllSafe,
    selectNone,
    setDryRun,
    setPermanentDelete,
    removeItemsByPaths,
    setLastError,
  } = useSweepStore();

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  useEffect(() => {
    void initPlatform();
    void attachScanListeners();
    return () => {
      void detachScanListeners();
    };
  }, []);

  const selectedItems = useMemo(
    () => items.filter((i) => selected.has(i.id) && i.deletable),
    [items, selected],
  );

  const selectedBytes = useMemo(
    () => selectedItems.reduce((sum, i) => sum + i.size_bytes, 0),
    [selectedItems],
  );

  const totalBytes = useMemo(
    () => items.reduce((sum, i) => sum + i.size_bytes, 0),
    [items],
  );

  async function handleConfirm() {
    setBusy(true);
    setStatusMsg(null);
    try {
      const result = await deleteSelected(
        selectedItems.map((i) => i.path),
        dryRun,
        permanentDelete,
      );
      if (dryRun) {
        setStatusMsg(
          `Dry-run: حدود ${formatBytes(result.freed_bytes)} قابل آزادسازی است.`,
        );
      } else {
        const okPaths = result.results
          .filter((r) => r.success)
          .map((r) => r.path);
        removeItemsByPaths(okPaths);
        setStatusMsg(
          `آزاد شد: ${formatBytes(result.freed_bytes)} · ناموفق: ${result.failed}`,
        );
      }
      setConfirmOpen(false);
    } catch (e) {
      setLastError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-shell">
      <div className="bg-glow" aria-hidden />

      <header className="topbar">
        <div className="brand-block">
          <div className="brand">Sweepr</div>
          <p className="tagline">
            اسکنر و پاک‌کنندهٔ فایل‌های زائد — هیچ چیزی بدون تأیید شما حذف
            نمی‌شود.
          </p>
        </div>
        <div className="platform-pill">
          پلتفرم: <strong>{platform || "…"}</strong>
        </div>
      </header>

      <section className="toolbar">
        <div className="toolbar-left">
          <button
            type="button"
            className="btn primary"
            onClick={() => void startScan()}
            disabled={scanning}
          >
            {scanning ? "در حال اسکن…" : items.length ? "اسکن مجدد" : "شروع اسکن"}
          </button>
          <button
            type="button"
            className="btn"
            onClick={selectAllSafe}
            disabled={scanning || items.length === 0}
          >
            انتخاب همهٔ ایمن
          </button>
          <button
            type="button"
            className="btn ghost"
            onClick={selectNone}
            disabled={selected.size === 0}
          >
            لغو انتخاب
          </button>
          <button
            type="button"
            className="btn danger"
            onClick={() => setConfirmOpen(true)}
            disabled={selectedItems.length === 0 || scanning}
          >
            حذف انتخاب‌شده
          </button>
        </div>

        <div className="toolbar-right">
          <label className="check">
            <input
              type="checkbox"
              checked={dryRun}
              onChange={(e) => setDryRun(e.target.checked)}
            />
            حالت Dry-run
          </label>
          <label className="check warn">
            <input
              type="checkbox"
              checked={permanentDelete}
              onChange={(e) => setPermanentDelete(e.target.checked)}
              disabled={dryRun}
            />
            حذف دائمی
          </label>
        </div>
      </section>

      <section className="overview">
        <div className="stat-card">
          <span className="stat-label">فضای یافت‌شده</span>
          <span className="stat-value">{formatBytes(totalBytes)}</span>
        </div>
        <div className="stat-card">
          <span className="stat-label">موارد</span>
          <span className="stat-value">{items.length}</span>
        </div>
        <div className="stat-card">
          <span className="stat-label">انتخاب‌شده</span>
          <span className="stat-value">
            {selectedItems.length} · {formatBytes(selectedBytes)}
          </span>
        </div>
        <div className="stat-card wide">
          <span className="stat-label">توزیع حجم بر اساس دسته</span>
          <CategoryChart items={items} />
        </div>
      </section>

      {(scanning || progress) && (
        <div className="progress-bar" role="status">
          {scanning ? (
            <>
              در حال اسکن
              {progress?.current_path ? `: ${progress.current_path}` : "…"}
              {progress
                ? ` · ${progress.items_found} مورد · ${formatBytes(progress.bytes_found)}`
                : ""}
            </>
          ) : null}
        </div>
      )}

      {complete && !scanning && (
        <div className="status ok">
          اسکن تمام شد در {complete.duration_ms}ms — {complete.item_count} مورد،{" "}
          {formatBytes(complete.total_bytes)}
          {complete.errors.length > 0
            ? ` · ${complete.errors.length} هشدار/خطای دسترسی`
            : ""}
        </div>
      )}

      {(lastError || statusMsg) && (
        <div className={`status ${lastError ? "err" : "ok"}`}>
          {lastError ?? statusMsg}
        </div>
      )}

      <ResultsList
        items={items}
        selected={selected}
        onToggle={toggleSelect}
      />

      <footer className="footer">
        قوانین اسکن از فایل JSON خوانده می‌شوند — مسیرهای سیستمی حیاتی هرگز در
        لیست حذف قرار نمی‌گیرند. پیش‌فرض حذف: سطل زباله.
      </footer>

      <ConfirmDialog
        open={confirmOpen}
        items={selectedItems}
        totalBytes={selectedBytes}
        dryRun={dryRun}
        permanent={permanentDelete}
        busy={busy}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={() => void handleConfirm()}
      />
    </div>
  );
}

export default App;
