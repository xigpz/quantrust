/**
 * SectorAnomalyAlert - 板块异动提醒组件
 * 监控板块异动并在发现时弹出提醒
 * 支持拖拽移动
 */
import { useState, useEffect, useRef } from 'react';
import { useSectors, formatPercent, formatNumber, getChangeColor } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';
import { AlertTriangle, X, TrendingUp, TrendingDown, Zap, GripHorizontal } from 'lucide-react';

interface SectorAnomaly {
  code: string;
  name: string;
  change_pct: number;
  turnover: number;
  reason: string;
}

interface SectorAnomalyAlertProps {
  onClose?: () => void;
}

export default function SectorAnomalyAlert({ onClose }: SectorAnomalyAlertProps) {
  const { data: sectors } = useSectors();
  const { openStock, switchTab } = useStockClick();
  const [anomalies, setAnomalies] = useState<SectorAnomaly[]>([]);
  const [isMinimized, setIsMinimized] = useState(false);
  const prevSectorsRef = useRef<Map<string, number>>(new Map());

  // 拖拽状态 - 默认显示在右下角
  const [position, setPosition] = useState({ x: 20, y: 20 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0 });
  const positionStartRef = useRef({ x: 0, y: 0 });

  // 检测板块异动
  useEffect(() => {
    if (!sectors || sectors.length === 0) return;

    const newAnomalies: SectorAnomaly[] = [];
    const currentSectors = new Map<string, number>();

    sectors.forEach((sector) => {
      currentSectors.set(sector.code, sector.change_pct);

      const prevChange = prevSectorsRef.current.get(sector.code);
      const avgTurnover = sectors.reduce((sum, s) => sum + s.turnover, 0) / sectors.length;

      if (prevChange !== undefined) {
        const changeDiff = sector.change_pct - prevChange;

        if (sector.change_pct > 5 && changeDiff > 1) {
          newAnomalies.push({
            code: sector.code,
            name: sector.name,
            change_pct: sector.change_pct,
            turnover: sector.turnover,
            reason: `快速上涨 ${changeDiff.toFixed(1)}%`,
          });
        } else if (sector.change_pct < -3 && changeDiff < -1) {
          newAnomalies.push({
            code: sector.code,
            name: sector.name,
            change_pct: sector.change_pct,
            turnover: sector.turnover,
            reason: `快速下跌 ${Math.abs(changeDiff).toFixed(1)}%`,
          });
        }
      } else {
        if (sector.change_pct > 7) {
          newAnomalies.push({
            code: sector.code,
            name: sector.name,
            change_pct: sector.change_pct,
            turnover: sector.turnover,
            reason: '高涨幅板块',
          });
        }
      }

      if (sector.turnover > avgTurnover * 2 && sector.change_pct > 3) {
        const existing = newAnomalies.find((a) => a.code === sector.code);
        if (!existing) {
          newAnomalies.push({
            code: sector.code,
            name: sector.name,
            change_pct: sector.change_pct,
            turnover: sector.turnover,
            reason: '资金大幅流入',
          });
        }
      }
    });

    prevSectorsRef.current = currentSectors;

    if (newAnomalies.length > 0) {
      setAnomalies((prev) => {
        const existingCodes = new Set(prev.map((a) => a.code));
        const uniqueNew = newAnomalies.filter((a) => !existingCodes.has(a.code));
        return [...prev, ...uniqueNew].slice(-5);
      });
    }
  }, [sectors]);

  const removeAnomaly = (code: string) => {
    setAnomalies((prev) => prev.filter((a) => a.code !== code));
  };

  // 拖拽处理
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    dragStartRef.current = { x: e.clientX, y: e.clientY };
    positionStartRef.current = { x: position.x, y: position.y };
  };

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - dragStartRef.current.x;
      const deltaY = e.clientY - dragStartRef.current.y;
      setPosition({
        x: Math.max(0, positionStartRef.current.x + deltaX),
        y: Math.max(0, positionStartRef.current.y + deltaY),
      });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging]);

  if (anomalies.length === 0) return null;

  return (
    <div
      className="fixed z-40 flex flex-col gap-2 max-w-sm"
      style={{ left: position.x, top: position.y }}
    >
      {/* 拖拽手柄 */}
      <div
        className={`flex items-center gap-2 px-3 py-1.5 bg-purple-600 text-white rounded-t-lg cursor-move select-none ${
          isDragging ? 'opacity-80' : ''
        }`}
        onMouseDown={handleMouseDown}
      >
        <GripHorizontal className="w-4 h-4 opacity-60" />
        <span className="text-sm font-medium">板块异动提醒</span>
        <div className="flex items-center gap-1 ml-auto">
          <button
            onClick={(e) => { e.stopPropagation(); setIsMinimized(!isMinimized); }}
            className="p-1 hover:bg-purple-500 rounded"
            title={isMinimized ? "展开" : "最小化"}
          >
            <TrendingDown className="w-3 h-3" />
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); onClose?.(); }}
            className="p-1 hover:bg-purple-500 rounded"
            title="关闭"
          >
            <X className="w-3 h-3" />
          </button>
        </div>
      </div>

      {!isMinimized && (
        <div className="space-y-2">
          {anomalies.map((anomaly) => (
            <div
              key={anomaly.code}
              className={`
                p-3 rounded-lg border shadow-lg cursor-pointer transition-all hover:scale-[1.02] relative
                ${anomaly.change_pct > 0
                  ? 'bg-red-950/50 border-red-800/50 hover:border-red-500'
                  : 'bg-green-950/50 border-green-800/50 hover:border-green-500'
                }
              `}
              onClick={() => {
                switchTab('sectors');
                removeAnomaly(anomaly.code);
              }}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className={`w-4 h-4 ${anomaly.change_pct > 0 ? 'text-red-400' : 'text-green-400'}`} />
                    <span className="font-medium text-sm text-foreground truncate">{anomaly.name}</span>
                  </div>
                  <div className="text-xs text-muted-foreground mt-1">{anomaly.reason}</div>
                </div>
                <div className="text-right shrink-0 ml-2">
                  <div className={`font-mono-data font-bold ${getChangeColor(anomaly.change_pct)}`}>
                    {formatPercent(anomaly.change_pct)}
                  </div>
                  <div className="text-[10px] text-muted-foreground">{formatNumber(anomaly.turnover)}</div>
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  removeAnomaly(anomaly.code);
                }}
                className="absolute top-1 right-1 p-1 opacity-50 hover:opacity-100"
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      )}

      {isMinimized && (
        <button
          onClick={() => setIsMinimized(false)}
          className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg shadow-lg animate-pulse"
        >
          <Zap className="w-4 h-4" />
          <span className="text-sm font-medium">{anomalies.length} 个板块异动</span>
          <TrendingUp className="w-4 h-4" />
        </button>
      )}
    </div>
  );
}
