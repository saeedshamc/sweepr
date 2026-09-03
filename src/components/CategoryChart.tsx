import {
  Bar,
  BarChart,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { CATEGORY_LABELS, type ScanItem } from "../types";
import { formatBytes } from "../lib/format";

const COLORS = [
  "#0d7377",
  "#14919b",
  "#2a9d8f",
  "#e9c46a",
  "#f4a261",
  "#e76f51",
  "#264653",
  "#6c757d",
];

interface Props {
  items: ScanItem[];
}

export function CategoryChart({ items }: Props) {
  const map = new Map<string, number>();
  for (const item of items) {
    map.set(item.category, (map.get(item.category) ?? 0) + item.size_bytes);
  }

  const data = [...map.entries()]
    .map(([category, size]) => ({
      category: CATEGORY_LABELS[category] ?? category,
      size,
    }))
    .sort((a, b) => b.size - a.size);

  if (data.length === 0) {
    return (
      <div className="chart-empty">
        هنوز نتیجه‌ای برای نمودار نیست — اسکن را شروع کنید.
      </div>
    );
  }

  return (
    <div className="chart-wrap">
      <ResponsiveContainer width="100%" height={220}>
        <BarChart data={data} margin={{ top: 8, right: 8, left: 8, bottom: 8 }}>
          <XAxis
            dataKey="category"
            tick={{ fill: "#4a5568", fontSize: 11 }}
            interval={0}
            angle={-18}
            textAnchor="end"
            height={56}
          />
          <YAxis
            tickFormatter={(v) => formatBytes(Number(v))}
            width={64}
            tick={{ fill: "#4a5568", fontSize: 11 }}
          />
          <Tooltip
            formatter={(value) => formatBytes(Number(value ?? 0))}
            contentStyle={{
              background: "#fff",
              border: "1px solid #d8e2e0",
              borderRadius: 8,
              fontFamily: "IBM Plex Sans, sans-serif",
            }}
          />
          <Bar dataKey="size" radius={[6, 6, 0, 0]}>
            {data.map((_, i) => (
              <Cell key={i} fill={COLORS[i % COLORS.length]} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
