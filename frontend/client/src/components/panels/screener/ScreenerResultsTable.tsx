import { getScreenerFieldLabel, type ScreenerResultRow } from "@/lib/screener";

interface ScreenerResultsTableProps {
  rows: ScreenerResultRow[];
  columns: string[];
  totalCount: number;
  loading: boolean;
  onSelectStock: (symbol: string, name?: string) => void;
  onAddToWatchlist: (symbol: string, name?: string) => void;
}

function formatValue(value: ScreenerResultRow[string]) {
  if (typeof value === "number") {
    return Number.isInteger(value) ? value.toLocaleString() : value.toFixed(2);
  }

  if (Array.isArray(value)) {
    return value.join(", ");
  }

  return value ?? "--";
}

export default function ScreenerResultsTable({ rows, columns, totalCount, loading, onSelectStock, onAddToWatchlist }: ScreenerResultsTableProps) {
  const visibleColumns = columns.length > 0 ? columns : Object.keys(rows[0] ?? {});

  return (
    <div className="flex h-full flex-col rounded-2xl border border-border bg-card/80">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div>
          <div className="text-sm font-semibold text-foreground">结果</div>
          <div className="text-xs text-muted-foreground">{loading ? "正在运行选股..." : `基于缓存行情命中 ${totalCount} 只股票`}</div>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {rows.length === 0 ? (
          <div className="p-6 text-sm text-muted-foreground">暂无结果。先配置筛选条件，再运行选股查看命中股票。</div>
        ) : (
          <table className="min-w-full text-sm">
            <thead className="sticky top-0 bg-card/95 text-left text-xs uppercase tracking-wide text-muted-foreground">
              <tr>
                {visibleColumns.map((column) => (
                  <th key={column} className="border-b border-border px-3 py-2 font-medium">
                    {getScreenerFieldLabel(column)}
                  </th>
                ))}
                <th className="border-b border-border px-3 py-2 font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, index) => {
                const symbol = typeof row.symbol === "string" ? row.symbol : undefined;
                const name = typeof row.name === "string" ? row.name : undefined;
                return (
                  <tr key={`${symbol ?? "row"}-${index}`} className="border-b border-border/60 text-foreground">
                    {visibleColumns.map((column) => (
                      <td
                        key={column}
                        className="px-3 py-2"
                        onClick={() => {
                          if (symbol) {
                            onSelectStock(symbol, name);
                          }
                        }}
                      >
                        {formatValue(row[column])}
                      </td>
                    ))}
                    <td className="px-3 py-2">
                      <button
                        type="button"
                        disabled={!symbol}
                        onClick={() => {
                          if (symbol) {
                            onAddToWatchlist(symbol, name);
                          }
                        }}
                        className="rounded-md border border-border px-2 py-1 text-xs text-foreground disabled:cursor-not-allowed disabled:text-muted-foreground"
                      >
                        自选
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
