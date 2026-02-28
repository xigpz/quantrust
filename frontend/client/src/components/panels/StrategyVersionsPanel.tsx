/**
 * StrategyVersionsPanel - 策略版本管理面板
 */
import { useState, useEffect } from 'react';
import { GitBranch, Plus, Clock, Eye, RotateCcw, FileCode } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface Strategy {
  id: string;
  name: string;
  code: string;
}

interface StrategyVersion {
  id: string;
  strategy_id: string;
  version: number;
  code: string;
  description: string | null;
  created_at: string;
}

export default function StrategyVersionsPanel() {
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<Strategy | null>(null);
  const [versions, setVersions] = useState<StrategyVersion[]>([]);
  const [selectedVersion, setSelectedVersion] = useState<StrategyVersion | null>(null);
  const [loading, setLoading] = useState(false);
  const [showCode, setShowCode] = useState(false);

  // 加载策略列表
  useEffect(() => {
    fetch('/api/strategies')
      .then(r => r.json())
      .then(data => {
        if (data.data) setStrategies(data.data);
      })
      .catch(console.error);
  }, []);

  // 加载版本历史
  const loadVersions = async (strategyId: string) => {
    setLoading(true);
    try {
      const res = await fetch(`/api/strategies/${strategyId}/versions`);
      const data = await res.json();
      if (data.success) {
        setVersions(data.data);
      }
    } catch (e) {
      console.error('Failed to load versions:', e);
    }
    setLoading(false);
  };

  // 选择策略
  const handleSelectStrategy = (strategy: Strategy) => {
    setSelectedStrategy(strategy);
    setSelectedVersion(null);
    setShowCode(false);
    loadVersions(strategy.id);
  };

  // 创建新版本
  const createVersion = async () => {
    if (!selectedStrategy) return;
    
    const description = prompt('版本描述（可选）:');
    
    try {
      const res = await fetch(`/api/strategies/${selectedStrategy.id}/versions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code: selectedStrategy.code,
          description: description || null,
        }),
      });
      const data = await res.json();
      if (data.success) {
        toast.success(`创建版本 v${data.data.version} 成功`);
        loadVersions(selectedStrategy.id);
      } else {
        toast.error(data.message);
      }
    } catch (e) {
      toast.error('创建失败');
    }
  };

  // 恢复版本
  const restoreVersion = (version: StrategyVersion) => {
    if (!selectedStrategy) return;
    
    if (!confirm(`确定要恢复到 v${version.version} 吗？`)) return;
    
    // 更新当前策略代码
    setSelectedStrategy({ ...selectedStrategy, code: version.code });
    toast.success(`已恢复到 v${version.version}`);
  };

  // 格式化时间
  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString('zh-CN', { 
      month: 'short', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <GitBranch className="w-4 h-4 text-indigo-400" />
          <h2 className="text-sm font-medium">策略版本</h2>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* 左侧：策略列表 */}
        <div className="w-1/3 border-r border-border">
          <div className="p-2 border-b border-border">
            <div className="text-xs text-muted-foreground px-2 py-1">选择策略</div>
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            {strategies.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground text-xs">
                暂无策略
              </div>
            ) : (
              <div className="p-2 space-y-1">
                {strategies.map(s => (
                  <div
                    key={s.id}
                    onClick={() => handleSelectStrategy(s)}
                    className={`p-2 rounded cursor-pointer text-xs ${
                      selectedStrategy?.id === s.id 
                        ? 'bg-primary text-primary-foreground' 
                        : 'hover:bg-accent'
                    }`}
                  >
                    <div className="font-medium">{s.name}</div>
                    <div className="text-xs opacity-70 truncate">{s.code.slice(0, 50)}...</div>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
        </div>

        {/* 中间：版本历史 */}
        <div className="w-1/3 border-r border-border">
          <div className="p-2 border-b border-border flex justify-between items-center">
            <div className="text-xs text-muted-foreground px-2 py-1">版本历史</div>
            {selectedStrategy && (
              <button
                onClick={createVersion}
                className="text-xs bg-primary text-primary-foreground px-2 py-1 rounded flex items-center gap-1"
              >
                <Plus className="w-3 h-3" /> 新建版本
              </button>
            )}
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            {!selectedStrategy ? (
              <div className="p-4 text-center text-muted-foreground text-xs">
                请先选择策略
              </div>
            ) : versions.length === 0 ? (
              <div className="p-4 text-center text-muted-foreground text-xs">
                {loading ? '加载中...' : '暂无版本记录'}
              </div>
            ) : (
              <div className="p-2 space-y-1">
                {versions.map(v => (
                  <div
                    key={v.id}
                    onClick={() => { setSelectedVersion(v); setShowCode(false); }}
                    className={`p-2 rounded cursor-pointer text-xs ${
                      selectedVersion?.id === v.id 
                        ? 'bg-primary text-primary-foreground' 
                        : 'hover:bg-accent'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span className="font-medium">v{v.version}</span>
                      <span className="text-xs opacity-70">{formatTime(v.created_at)}</span>
                    </div>
                    {v.description && (
                      <div className="text-xs opacity-70 truncate">{v.description}</div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
        </div>

        {/* 右侧：版本详情 */}
        <div className="flex-1">
          <div className="p-2 border-b border-border flex justify-between items-center">
            <div className="text-xs text-muted-foreground px-2 py-1">版本详情</div>
            {selectedVersion && (
              <div className="flex gap-1">
                <button
                  onClick={() => setShowCode(!showCode)}
                  className="text-xs bg-secondary px-2 py-1 rounded flex items-center gap-1"
                >
                  <Eye className="w-3 h-3" /> {showCode ? '隐藏' : '查看'}
                </button>
                <button
                  onClick={() => restoreVersion(selectedVersion)}
                  className="text-xs bg-green-600 text-white px-2 py-1 rounded flex items-center gap-1"
                >
                  <RotateCcw className="w-3 h-3" /> 恢复
                </button>
              </div>
            )}
          </div>
          <ScrollArea className="h-[calc(100%-40px)]">
            {!selectedVersion ? (
              <div className="p-4 text-center text-muted-foreground text-xs">
                请选择要查看的版本
              </div>
            ) : showCode ? (
              <pre className="p-4 text-xs text-green-400 font-mono whitespace-pre-wrap overflow-auto">
                {selectedVersion.code}
              </pre>
            ) : (
              <div className="p-4 space-y-3">
                <div>
                  <div className="text-xs text-muted-foreground">版本号</div>
                  <div className="text-sm font-medium">v{selectedVersion.version}</div>
                </div>
                <div>
                  <div className="text-xs text-muted-foreground">创建时间</div>
                  <div className="text-sm">{new Date(selectedVersion.created_at).toLocaleString()}</div>
                </div>
                <div>
                  <div className="text-xs text-muted-foreground">描述</div>
                  <div className="text-sm">{selectedVersion.description || '-'}</div>
                </div>
                <div>
                  <div className="text-xs text-muted-foreground">代码预览</div>
                  <div className="text-xs bg-gray-800 p-2 rounded mt-1 font-mono text-green-400 max-h-32 overflow-auto">
                    {selectedVersion.code.slice(0, 200)}...
                  </div>
                </div>
              </div>
            )}
          </ScrollArea>
        </div>
      </div>
    </div>
  );
}
