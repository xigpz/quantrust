/**
 * RiskPanel - 风险控制面板
 */
import { useState } from 'react';
import { Shield, Save, RefreshCw, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';

interface RiskConfig {
  max_position_ratio: number;
  max_single_position: number;
  stop_loss_ratio: number;
  take_profit_ratio: number;
  max_drawdown_threshold: number;
  enabled: boolean;
}

interface RiskCheckResult {
  allowed: boolean;
  reason: string;
  stop_loss_price: number | null;
  take_profit_price: number | null;
  max_quantity: number;
}

export default function RiskPanel() {
  const [config, setConfig] = useState<RiskConfig>({
    max_position_ratio: 0.8,
    max_single_position: 0.2,
    stop_loss_ratio: 0.05,
    take_profit_ratio: 0.15,
    max_drawdown_threshold: 0.15,
    enabled: true,
  });

  const [checkResult, setCheckResult] = useState<RiskCheckResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  // 加载配置
  const loadConfig = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/risk/config');
      const data = await res.json();
      if (data.success) {
        setConfig(data.data);
      }
    } catch (e) {
      console.error('Failed to load config:', e);
    }
    setLoading(false);
  };

  // 保存配置
  const saveConfig = async () => {
    setSaving(true);
    try {
      const res = await fetch('/api/risk/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      const data = await res.json();
      if (data.success) {
        alert('保存成功！');
      }
    } catch (e) {
      console.error('Failed to save config:', e);
    }
    setSaving(false);
  };

  // 测试风控检查
  const testCheck = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/risk/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'buy',
          symbol: '600519',
          entry_price: 1800,
          current_price: 1850,
          quantity: 100,
          current_position: 50000,
          total_capital: 100000,
        }),
      });
      const data = await res.json();
      if (data.success) {
        setCheckResult(data.data);
      }
    } catch (e) {
      console.error('Failed to check risk:', e);
    }
    setLoading(false);
  };

  const updateConfig = (key: keyof RiskConfig, value: number | boolean) => {
    setConfig(prev => ({ ...prev, [key]: value }));
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Shield className="w-4 h-4 text-blue-400" />
          <h2 className="text-sm font-semibold">风险控制</h2>
        </div>
        <div className="flex items-center gap-2">
          <button onClick={loadConfig} className="text-muted-foreground hover:text-foreground transition-colors">
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-4 space-y-6">
          {/* 风控开关 */}
          <div className="flex items-center justify-between bg-card rounded-lg p-4 border border-border">
            <div className="flex items-center gap-2">
              {config.enabled ? (
                <CheckCircle className="w-5 h-5 text-green-400" />
              ) : (
                <XCircle className="w-5 h-5 text-red-400" />
              )}
              <span className="font-medium">风控开关</span>
            </div>
            <button
              onClick={() => updateConfig('enabled', !config.enabled)}
              className={`w-12 h-6 rounded-full transition-colors ${
                config.enabled ? 'bg-green-500' : 'bg-gray-600'
              }`}
            >
              <div className={`w-5 h-5 rounded-full bg-white transition-transform ${
                config.enabled ? 'translate-x-6' : 'translate-x-0.5'
              }`} />
            </button>
          </div>

          {/* 风控参数 */}
          <div className="bg-card rounded-lg p-4 border border-border space-y-4">
            <h3 className="font-medium text-sm">风控参数</h3>
            
            {/* 最大仓位 */}
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-muted-foreground">最大仓位</span>
                <span>{(config.max_position_ratio * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                min="0.1"
                max="1"
                step="0.1"
                value={config.max_position_ratio}
                onChange={(e) => updateConfig('max_position_ratio', parseFloat(e.target.value))}
                className="w-full"
              />
            </div>

            {/* 单股最大仓位 */}
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-muted-foreground">单股最大仓位</span>
                <span>{(config.max_single_position * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                min="0.05"
                max="0.5"
                step="0.05"
                value={config.max_single_position}
                onChange={(e) => updateConfig('max_single_position', parseFloat(e.target.value))}
                className="w-full"
              />
            </div>

            {/* 止损比例 */}
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-muted-foreground">止损比例</span>
                <span className="text-red-400">-{(config.stop_loss_ratio * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                min="0.01"
                max="0.15"
                step="0.01"
                value={config.stop_loss_ratio}
                onChange={(e) => updateConfig('stop_loss_ratio', parseFloat(e.target.value))}
                className="w-full"
              />
            </div>

            {/* 止盈比例 */}
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-muted-foreground">止盈比例</span>
                <span className="text-green-400">+{(config.take_profit_ratio * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                min="0.05"
                max="0.5"
                step="0.05"
                value={config.take_profit_ratio}
                onChange={(e) => updateConfig('take_profit_ratio', parseFloat(e.target.value))}
                className="w-full"
              />
            </div>

            {/* 最大回撤 */}
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span className="text-muted-foreground">最大回撤阈值</span>
                <span className="text-yellow-400">-{(config.max_drawdown_threshold * 100).toFixed(0)}%</span>
              </div>
              <input
                type="range"
                min="0.05"
                max="0.3"
                step="0.05"
                value={config.max_drawdown_threshold}
                onChange={(e) => updateConfig('max_drawdown_threshold', parseFloat(e.target.value))}
                className="w-full"
              />
            </div>

            <button
              onClick={saveConfig}
              disabled={saving}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded flex items-center justify-center gap-2 disabled:opacity-50"
            >
              <Save className="w-4 h-4" />
              {saving ? '保存中...' : '保存配置'}
            </button>
          </div>

          {/* 测试风控 */}
          <div className="bg-card rounded-lg p-4 border border-border">
            <h3 className="font-medium text-sm mb-3">风控测试</h3>
            <button
              onClick={testCheck}
              disabled={loading}
              className="w-full bg-gray-700 hover:bg-gray-600 text-white py-2 px-4 rounded flex items-center justify-center gap-2 disabled:opacity-50"
            >
              <AlertTriangle className="w-4 h-4" />
              {loading ? '测试中...' : '测试买入风控'}
            </button>

            {checkResult && (
              <div className="mt-3 p-3 bg-gray-800 rounded text-sm">
                <div className="flex items-center gap-2 mb-2">
                  {checkResult.allowed ? (
                    <CheckCircle className="w-4 h-4 text-green-400" />
                  ) : (
                    <XCircle className="w-4 h-4 text-red-400" />
                  )}
                  <span className={checkResult.allowed ? 'text-green-400' : 'text-red-400'}>
                    {checkResult.allowed ? '允许买入' : '禁止买入'}
                  </span>
                </div>
                <div className="text-muted-foreground text-xs">
                  {checkResult.reason}
                </div>
                {checkResult.stop_loss_price && (
                  <div className="mt-2 text-xs">
                    <span className="text-red-400">止损价: </span>
                    <span className="text-red-400">¥{checkResult.stop_loss_price.toFixed(2)}</span>
                  </div>
                )}
                {checkResult.take_profit_price && (
                  <div className="text-xs">
                    <span className="text-green-400">止盈价: </span>
                    <span className="text-green-400">¥{checkResult.take_profit_price.toFixed(2)}</span>
                  </div>
                )}
                <div className="text-xs mt-1">
                  <span className="text-muted-foreground">最大可买: </span>
                  <span>{checkResult.max_quantity}股</span>
                </div>
              </div>
            )}
          </div>

          {/* 说明 */}
          <div className="text-xs text-muted-foreground text-center">
            风控参数仅对模拟交易生效，实盘需券商配置
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
