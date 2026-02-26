/**
 * SettingsPanel - 设置面板
 */
import { useState } from 'react';
import { Settings, Server, Database, Wifi, Github, Info } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';

export default function SettingsPanel() {
  const [apiUrl, setApiUrl] = useState(import.meta.env.VITE_API_BASE || 'http://localhost:8080');
  const [refreshInterval, setRefreshInterval] = useState(15);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border">
        <Settings className="w-4 h-4 text-muted-foreground" />
        <h2 className="text-sm font-semibold">系统设置</h2>
      </div>

      <div className="p-4 space-y-6 overflow-auto flex-1">
        {/* Connection */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
            <Server className="w-3 h-3" /> 后端连接
          </h3>
          <div className="bg-card rounded-lg p-3 border border-border/50 space-y-2">
            <label className="text-[10px] text-muted-foreground">API 地址</label>
            <Input
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              className="h-8 text-xs font-mono-data bg-background"
            />
            <label className="text-[10px] text-muted-foreground">刷新间隔 (秒)</label>
            <Input
              type="number"
              value={refreshInterval}
              onChange={(e) => setRefreshInterval(Number(e.target.value))}
              className="h-8 text-xs font-mono-data bg-background"
              min={5}
              max={300}
            />
            <Button
              variant="outline"
              size="sm"
              className="w-full text-xs h-7"
              onClick={() => toast('设置已保存', { description: '重新加载页面后生效' })}
            >
              保存设置
            </Button>
          </div>
        </div>

        {/* System Info */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
            <Info className="w-3 h-3" /> 系统信息
          </h3>
          <div className="bg-card rounded-lg p-3 border border-border/50 space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-muted-foreground">前端版本</span>
              <span className="font-mono-data">v0.1.0</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">后端框架</span>
              <span className="font-mono-data">Rust / Axum</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">数据库</span>
              <span className="font-mono-data">SQLite</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">数据源</span>
              <span className="font-mono-data">东方财富 API</span>
            </div>
          </div>
        </div>

        {/* About */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
            <Github className="w-3 h-3" /> 关于
          </h3>
          <div className="bg-card rounded-lg p-3 border border-border/50 text-xs text-muted-foreground">
            <p className="mb-2">
              <span className="text-foreground font-semibold">QuantRust</span> 是一个开源的A股量化交易工具，
              后端使用 Rust 构建，前端使用 React + TypeScript。
            </p>
            <p>
              支持实时行情监测、热点股票追踪、异动检测、板块分析、资金流向、策略回测等功能。
            </p>
            <a
              href="https://github.com/xigpz/quantrust"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-primary hover:underline mt-2"
            >
              <Github className="w-3 h-3" /> GitHub 仓库
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
