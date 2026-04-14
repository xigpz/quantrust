/**
 * GlobalSearch - 全局搜索组件
 * 支持 Ctrl+K 快捷键打开
 */
import { useState, useEffect, useCallback, useRef } from 'react';
import { Search, X, TrendingUp, Clock, Star, Plus, Check, Loader2 } from 'lucide-react';
import { useSearch, useWatchlist, formatPrice, formatPercent, getChangeColor, addToWatchlist, removeFromWatchlist } from '@/hooks/useMarketData';
import { useStockClick } from '@/pages/Dashboard';
import { Dialog, DialogContent } from '@/components/ui/dialog';
import { toast } from 'sonner';

interface GlobalSearchProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export default function GlobalSearch({ open, onOpenChange }: GlobalSearchProps) {
  const [query, setQuery] = useState('');
  const [recentSearches, setRecentSearches] = useState<string[]>([]);
  const [watchlist, setWatchlist] = useState<string[]>([]);
  const [addingSymbol, setAddingSymbol] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const { openStock, switchTab } = useStockClick();

  const { data: results, loading } = useSearch(query);
  const { data: watchlistData } = useWatchlist?.() || { data: null };

  // Load watchlist when opened
  useEffect(() => {
    if (open && watchlistData) {
      setWatchlist(watchlistData.map((w: any) => w.symbol));
    }
  }, [open, watchlistData]);

  // Check if a symbol is in watchlist
  const isInWatchlist = (symbol: string) => watchlist.includes(symbol);

  // Handle add/remove from watchlist
  const handleToggleWatchlist = async (e: React.MouseEvent, symbol: string, name: string) => {
    e.stopPropagation();
    if (isInWatchlist(symbol)) {
      try {
        const res = await removeFromWatchlist(symbol);
        if (res.success) {
          setWatchlist(prev => prev.filter(s => s !== symbol));
          toast.success('已移除自选', { description: `${name} (${symbol})` });
        }
      } catch {
        toast.error('操作失败');
      }
    } else {
      try {
        setAddingSymbol(symbol);
        const res = await addToWatchlist({ symbol, name });
        if (res.success) {
          setWatchlist(prev => [...prev, symbol]);
          toast.success('已添加自选', { description: `${name} (${symbol})` });
        }
      } catch {
        toast.error('操作失败');
      } finally {
        setAddingSymbol(null);
      }
    }
  };

  // 从localStorage加载最近搜索
  useEffect(() => {
    const saved = localStorage.getItem('recent_searches');
    if (saved) {
      setRecentSearches(JSON.parse(saved));
    }
  }, []);

  // 保存最近搜索
  const saveRecentSearch = useCallback((symbol: string) => {
    const updated = [symbol, ...recentSearches.filter(s => s !== symbol)].slice(0, 10);
    setRecentSearches(updated);
    localStorage.setItem('recent_searches', JSON.stringify(updated));
  }, [recentSearches]);

  // 处理选择
  const handleSelect = (symbol: string, name: string) => {
    saveRecentSearch(symbol);
    onOpenChange(false);
    setQuery('');
    openStock(symbol, name);
  };

  // 输入框聚焦
  useEffect(() => {
    if (open && inputRef.current) {
      inputRef.current.focus();
    }
  }, [open]);

  // 清空搜索
  const handleClear = () => {
    setQuery('');
    inputRef.current?.focus();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl p-0 gap-0 overflow-hidden">
        {/* 搜索框 */}
        <div className="flex items-center gap-3 px-4 py-3 border-b">
          <Search className="w-5 h-5 text-muted-foreground shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索股票代码或名称... (Ctrl+K)"
            className="flex-1 bg-transparent outline-none text-lg placeholder:text-muted-foreground"
          />
          {query && (
            <button onClick={handleClear} className="p-1 hover:bg-muted rounded">
              <X className="w-4 h-4 text-muted-foreground" />
            </button>
          )}
        </div>

        {/* 搜索结果 / 最近搜索 */}
        <div className="max-h-96 overflow-y-auto">
          {loading ? (
            <div className="p-8 text-center text-muted-foreground">
              搜索中...
            </div>
          ) : query && results && results.length > 0 ? (
            <div className="py-2">
              <div className="px-4 py-1 text-xs text-muted-foreground">
                搜索结果 ({results.length})
              </div>
              {results.slice(0, 20).map((stock) => (
                <div
                  key={stock.symbol}
                  onClick={() => handleSelect(stock.symbol, stock.name)}
                  className="flex items-center justify-between px-4 py-3 hover:bg-accent cursor-pointer transition-colors group"
                >
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <TrendingUp className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <div className="font-medium">{stock.name}</div>
                      <div className="text-sm text-muted-foreground font-mono">{stock.symbol}</div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="text-right">
                      <div className="font-mono">{formatPrice(stock.price)}</div>
                      <div className={`text-sm font-mono ${getChangeColor(stock.change_pct)}`}>
                        {formatPercent(stock.change_pct)}
                      </div>
                    </div>
                    <button
                      onClick={(e) => handleToggleWatchlist(e, stock.symbol, stock.name)}
                      disabled={addingSymbol === stock.symbol}
                      className={`p-1.5 rounded-md transition-colors ${
                        isInWatchlist(stock.symbol)
                          ? 'text-yellow-400 hover:bg-yellow-400/10'
                          : 'text-muted-foreground opacity-0 group-hover:opacity-100 hover:bg-muted'
                      }`}
                      title={isInWatchlist(stock.symbol) ? '移除自选' : '添加自选'}
                    >
                      {addingSymbol === stock.symbol ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : isInWatchlist(stock.symbol) ? (
                        <Check className="w-4 h-4" />
                      ) : (
                        <Plus className="w-4 h-4" />
                      )}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : query && results?.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground">
              未找到相关股票
            </div>
          ) : (
            <div className="py-2">
              {recentSearches.length > 0 && (
                <>
                  <div className="px-4 py-1 text-xs text-muted-foreground flex items-center gap-1">
                    <Clock className="w-3 h-3" /> 最近搜索
                  </div>
                  {recentSearches.slice(0, 5).map((symbol) => (
                    <div
                      key={symbol}
                      onClick={() => {
                        onOpenChange(false);
                        openStock(symbol);
                      }}
                      className="flex items-center gap-3 px-4 py-2 hover:bg-accent cursor-pointer transition-colors"
                    >
                      <Star className="w-4 h-4 text-muted-foreground" />
                      <span className="font-mono">{symbol}</span>
                    </div>
                  ))}
                </>
              )}
              {recentSearches.length === 0 && (
                <div className="p-8 text-center text-muted-foreground text-sm">
                  输入股票代码或名称搜索<br />
                  支持拼音首字母搜索
                </div>
              )}
            </div>
          )}
        </div>

        {/* 底部提示 */}
        <div className="px-4 py-2 border-t bg-muted/30 text-xs text-muted-foreground flex items-center gap-4">
          <span><kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">Enter</kbd> 选中</span>
          <span><kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">Esc</kbd> 关闭</span>
        </div>
      </DialogContent>
    </Dialog>
  );
}

// 快捷键hook
export function useGlobalSearch() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+K 或 Cmd+K 打开搜索
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setOpen(true);
      }
      // Esc 关闭搜索（如果没在其他地方处理）
      if (e.key === 'Escape') {
        setOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return { open, setOpen };
}
