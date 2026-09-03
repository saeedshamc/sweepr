import { formatBytes } from "../lib/format";
import type { ScanItem } from "../types";

interface Props {
  open: boolean;
  items: ScanItem[];
  totalBytes: number;
  dryRun: boolean;
  permanent: boolean;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function ConfirmDialog({
  open,
  items,
  totalBytes,
  dryRun,
  permanent,
  busy,
  onCancel,
  onConfirm,
}: Props) {
  if (!open) return null;

  return (
    <div className="modal-backdrop" role="presentation" onClick={onCancel}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="confirm-title">
          {dryRun ? "پیش‌نمایش حذف (Dry-run)" : "تأیید حذف"}
        </h2>
        <p className="modal-lead">
          {dryRun
            ? "هیچ فایلی حذف نمی‌شود — فقط میزان فضای قابل آزادسازی محاسبه می‌شود."
            : permanent
              ? "موارد انتخاب‌شده برای همیشه حذف می‌شوند و به Recycle Bin / Trash نمی‌روند."
              : "موارد انتخاب‌شده به Recycle Bin / Trash منتقل می‌شوند."}
        </p>

        <div className="modal-summary">
          <strong>{items.length}</strong> مورد ·{" "}
          <strong>{formatBytes(totalBytes)}</strong>
        </div>

        <ul className="confirm-list">
          {items.map((item) => (
            <li key={item.id}>
              <span className="confirm-name">{item.name}</span>
              <span className="mono">{formatBytes(item.size_bytes)}</span>
              <span className="item-path">{item.path}</span>
            </li>
          ))}
        </ul>

        <div className="modal-actions">
          <button type="button" className="btn ghost" onClick={onCancel} disabled={busy}>
            انصراف
          </button>
          <button
            type="button"
            className={dryRun ? "btn primary" : "btn danger"}
            onClick={onConfirm}
            disabled={busy}
          >
            {busy
              ? "در حال اجرا…"
              : dryRun
                ? "اجرای Dry-run"
                : permanent
                  ? "حذف دائمی"
                  : "ارسال به سطل زباله"}
          </button>
        </div>
      </div>
    </div>
  );
}
