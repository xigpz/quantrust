/**
 * SettingsPanel - 设置面板
 */
import { useState, useEffect } from 'react';
import { Settings, Server, Database, Wifi, Github, Info, Key, Eye, EyeOff } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';

export default function SettingsPanel() {
  const [apiUrl, setApiUrl] = useState(import.meta.env.VITE_API_BASE || 'http://localhost:8080');
  const [refreshInterval, setRefreshInterval] = useState(15);
  const [minimaxKey, setMinimaxKey] = useState('');
  const [minimaxKeySet, setMinimaxKeySet] = useState(false);
  const [showKey, setShowKey] = useState(false);

  useEffect(() => {
    fetch(`${apiUrl}/api/config`)
      .then(res => res.json())
      .then(data => {
        if (data.success) {
          setMinimaxKeySet(data.data.minimax_api_key_set);
        }
      })
      .catch(() => {});
  }, [apiUrl]);

  const saveMinimaxKey = async () => {
    try {
      const res = await fetch(`${apiUrl}/api/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ minimax_api_key: minimaxKey }),
      });
      const data = await res.json();
      if (data.success) {
        setMinimaxKeySet(data.data.minimax_api_key_set);
        toast.success(minimaxKey ? 'API Key 已保存' : 'API Key 已清除');
        setMinimaxKey('');
      } else {
        toast.error('保存失败');
      }
    } catch (e) {
      toast.error('保存失败');
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border">
        <Settings className="w-4 h-4 text-muted-foreground" />
        <h2 className="text-sm font-semibold">系统设置</h2>
      </div>

      <div className="p-4 space-y-6 overflow-auto flex-1">
        {/* MiniMax API Key */}
        <div className="space-y-3">
          <h3 className="text-xs font-semibold text-muted-foreground flex items-center gap-1.5">
            <Key className="w-3 h-3" /> AI 能力
          </h3>
          <div className="bg-card rounded-lg p-3 border border-border/50 space-y-2">
            <label className="text-[10px] text-muted-foreground">MiniMax API Key</label>
            <div className="flex gap-2">
              <Input
                type={showKey ? 'text' : 'password'}
                value={minimaxKey}
                onChange={(e) => setMinimaxKey(e.target.value)}
                placeholder={minimaxKeySet ? '已设置 (请重新输入以修改)' : '输入 API Key'}
                className="h-8 text-xs font-mono-data bg-background flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                className="h-8 px-2"
                onClick={() => setShowKey(!showKey)}
              >
                {showKey ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
              </Button>
            </div>
            <p className="text-[10px] text-muted-foreground">
              {minimaxKeySet ? '✅ API Key 已配置，AI形态分析将使用AI分析' : '⚠️ 未配置API Key，将使用规则分析'}
            </p>
            <Button
              variant="outline"
              size="sm"
              className="w-full text-xs h-7"
              onClick={saveMinimaxKey}
            >
              保存 API Key
            </Button>
          </div>
        </div>

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
