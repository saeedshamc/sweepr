import type { ScanItem } from "../types";
import { formatBytes, formatDate } from "../lib/format";

interface Props {
  item: ScanItem;
  selected: boolean;
  onToggle: () => void;
}

const RISK_LABEL: Record<string, string> = {
  safe: "ایمن",
  caution: "احتیاط",
  risky: "پرریسک",
};

export function ResultRow({ item, selected, onToggle }: Props) {
  const canSelect = item.deletable && !item.informational_only;

  return (
    <tr className={`result-row risk-${item.risk}`}>
      <td>
        <input
          type="checkbox"
          checked={selected}
          disabled={!canSelect}
          onChange={onToggle}
          aria-label={`Select ${item.name}`}
        />
      </td>
      <td>
        <div className="item-name">{item.name}</div>
        <div className="item-path" title={item.path}>
          {item.path}
        </div>
        {item.cleanup_hint && (
          <div className="item-hint">{item.cleanup_hint}</div>
        )}
      </td>
      <td className="mono">{formatBytes(item.size_bytes)}</td>
      <td className="mono muted">{item.file_count.toLocaleString()}</td>
      <td className="muted">{formatDate(item.last_modified)}</td>
      <td>
        <span className={`badge badge-${item.risk}`}>
          {RISK_LABEL[item.risk] ?? item.risk}
        </span>
      </td>
    </tr>
  );
}
