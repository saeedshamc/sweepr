import { useMemo } from "react";
import { CATEGORY_LABELS, type ScanItem } from "../types";
import { ResultRow } from "./ResultRow";

interface Props {
  items: ScanItem[];
  selected: Set<string>;
  onToggle: (id: string) => void;
}

export function ResultsList({ items, selected, onToggle }: Props) {
  const grouped = useMemo(() => {
    const map = new Map<string, ScanItem[]>();
    for (const item of items) {
      const list = map.get(item.category) ?? [];
      list.push(item);
      map.set(item.category, list);
    }
    return [...map.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  }, [items]);

  if (items.length === 0) {
    return (
      <div className="empty-state">
        هیچ موردی پیدا نشده. روی «شروع اسکن» بزنید تا مسیرهای تعریف‌شده در
        قوانین بررسی شوند.
      </div>
    );
  }

  return (
    <div className="results">
      {grouped.map(([category, group]) => (
        <section key={category} className="category-block">
          <header className="category-header">
            <h3>{CATEGORY_LABELS[category] ?? category}</h3>
            <span className="muted">{group.length} مورد</span>
          </header>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th style={{ width: 36 }} />
                  <th>نام / مسیر</th>
                  <th>حجم</th>
                  <th>فایل‌ها</th>
                  <th>آخرین تغییر</th>
                  <th>ریسک</th>
                </tr>
              </thead>
              <tbody>
                {group.map((item) => (
                  <ResultRow
                    key={item.id}
                    item={item}
                    selected={selected.has(item.id)}
                    onToggle={() => onToggle(item.id)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        </section>
      ))}
    </div>
  );
}
