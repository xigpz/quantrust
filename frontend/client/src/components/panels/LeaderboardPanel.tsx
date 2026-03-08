/**
 * LeaderboardPanel - 模拟交易排行榜面板
 */
import { useState, useEffect } from 'react';
import { Trophy, RefreshCw, User, TrendingUp, TrendingDown, Wallet, Activity, Target } from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

interface LeaderboardEntry {
  rank: number;
  username: string;
  current_capital: number;
  total_return: number;
  return_rate: number;
  total_trades: number;
  win_count: number;
  loss_count: number;
  win_rate: number;
  positions_count: number;
  updated_at: string;
}

interface MyStats {
  username: string;
  current_capital: number;
  total_return: number;
  return_rate: number;
  total_trades: number;
  positions_count: number;
  cash: number;
  positions_value: number;
}

const API_BASE = 'http://localhost:8082';

async function fetchLeaderboard(): Promise<LeaderboardEntry[]> {
  const res = await fetch(`${API_BASE}/api/sim/leaderboard`);
  const data = await res.json();
  return data.success ? data.data : [];
}

async function fetchMyStats(): Promise<MyStats | null> {
  try {
    const res = await fetch(`${API_BASE}/api/sim/user/stats`);
    const data = await res.json();
    return data.success ? data.data : null;
  } catch {
    return null;
  }
}

async function updateLeaderboard() {
  const res = await fetch(`${API_BASE}/api/sim/leaderboard/update`, { method: 'POST' });
  const data = await res.json();
  return data.success;
}

async function setUsername(username: string) {
  const res = await fetch(`${API_BASE}/api/sim/user/set`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username }),
  });
  const data = await res.json();
  return data.success;
}

function formatMoney(value: number): string {
  return value.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function formatPercent(value: number): string {
  return (value >= 0 ? '+' : '') + value.toFixed(2) + '%';
}

export default function LeaderboardPanel() {
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [myStats, setMyStats] = useState<MyStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [username, setUsernameInput] = useState('');
  const [showSetName, setShowSetName] = useState(false);

  const loadData = async () => {
    setLoading(true);
    try {
      const [board, stats] = await Promise.all([fetchLeaderboard(), fetchMyStats()]);
      setLeaderboard(board);
      setMyStats(stats);
      if (stats && !board.find(e => e.username === stats.username)) {
        // 当前用户不在榜上，但有交易，提示更新
      }
    } catch (e) {
      toast.error('加载排行榜失败', { description: '请检查后端服务' });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleUpdateMyRank = async () => {
    const success = await updateLeaderboard();
    if (success) {
      toast.success('排行榜已更新');
      loadData();
    } else {
      toast.error('更新失败');
    }
  };

  const handleSetUsername = async () => {
    if (!username.trim()) return;
    const success = await setUsername(username.trim());
    if (success) {
      toast.success(`用户名已设置为: ${username}`);
      setShowSetName(false);
      loadData();
    } else {
      toast.error('设置用户名失败');
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <Trophy className="w-4 h-4 text-yellow-400" />
          <h2 className="text-sm font-semibold">模拟交易排行榜</h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowSetName(!showSetName)}
            className="text-muted-foreground hover:text-foreground transition-colors"
            title="设置用户名"
          >
            <User className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={handleUpdateMyRank}
            className="text-muted-foreground hover:text-foreground transition-colors"
            title="更新我的排名"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Set Username Modal */}
      {showSetName && (
        <div className="px-4 py-3 bg-muted/50 border-b border-border">
          <div className="flex gap-2">
            <input
              type="text"
              value={username}
              onChange={(e) => setUsernameInput(e.target.value)}
              placeholder="输入用户名"
              className="flex-1 px-3 py-1.5 text-sm bg-background border border-border rounded-md focus:outline-none focus:ring-1 focus:ring-primary"
              onKeyDown={(e) => e.key === 'Enter' && handleSetUsername()}
            />
            <button
              onClick={handleSetUsername}
              className="px-3 py-1.5 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
            >
              设置
            </button>
          </div>
        </div>
      )}

      {/* My Stats Card */}
      {myStats && (
        <div className="px-4 py-3 bg-muted/30 border-b border-border">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <User className="w-4 h-4 text-primary" />
              <span className="text-sm font-medium">{myStats.username}</span>
            </div>
            <span className="text-xs text-muted-foreground">
              {new Date().toLocaleDateString('zh-CN')}
            </span>
          </div>
          <div className="grid grid-cols-4 gap-2 text-center">
            <div className="bg-background/50 rounded p-2">
              <div className="text-xs text-muted-foreground">总资产</div>
              <div className="text-sm font-semibold">{formatMoney(myStats.current_capital)}</div>
            </div>
            <div className="bg-background/50 rounded p-2">
              <div className="text-xs text-muted-foreground">收益率</div>
              <div className={`text-sm font-semibold ${myStats.return_rate >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                {formatPercent(myStats.return_rate)}
              </div>
            </div>
            <div className="bg-background/50 rounded p-2">
              <div className="text-xs text-muted-foreground">交易次数</div>
              <div className="text-sm font-semibold">{myStats.total_trades}</div>
            </div>
            <div className="bg-background/50 rounded p-2">
              <div className="text-xs text-muted-foreground">持仓</div>
              <div className="text-sm font-semibold">{myStats.positions_count}</div>
            </div>
          </div>
          <div className="flex gap-4 mt-2 text-xs text-muted-foreground">
            <span className="flex items-center gap-1">
              <Wallet className="w-3 h-3" />
              现金: {formatMoney(myStats.cash)}
            </span>
            <span className="flex items-center gap-1">
              <Activity className="w-3 h-3" />
              持仓: {formatMoney(myStats.positions_value)}
            </span>
          </div>
        </div>
      )}

      {/* Leaderboard Table */}
      <ScrollArea className="flex-1">
        {leaderboard.length > 0 ? (
          <table className="w-full text-xs">
            <thead className="sticky top-0 bg-card z-10">
              <tr className="text-muted-foreground border-b border-border">
                <th className="text-center py-2 px-2 font-medium w-12">排名</th>
                <th className="text-left py-2 px-2 font-medium">用户名</th>
                <th className="text-right py-2 px-2 font-medium">总资产</th>
                <th className="text-right py-2 px-2 font-medium">收益率</th>
                <th className="text-right py-2 px-2 font-medium">交易</th>
                <th className="text-right py-2 px-2 font-medium">胜率</th>
                <th className="text-right py-2 px-2 font-medium">持仓</th>
              </tr>
            </thead>
            <tbody>
              {leaderboard.map((entry) => (
                <tr
                  key={entry.username}
                  className={`border-b border-border/50 hover:bg-accent/50 transition-colors ${
                    myStats?.username === entry.username ? 'bg-primary/10' : ''
                  }`}
                >
                  <td className="text-center py-2 px-2">
                    {entry.rank <= 3 ? (
                      <span className={`inline-flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold ${
                        entry.rank === 1 ? 'bg-yellow-500/20 text-yellow-500' :
                        entry.rank === 2 ? 'bg-gray-400/20 text-gray-400' :
                        'bg-amber-700/20 text-amber-700'
                      }`}>
                        {entry.rank}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">{entry.rank}</span>
                    )}
                  </td>
                  <td className="py-2 px-2 font-medium">
                    <div className="flex items-center gap-1">
                      {entry.username}
                      {myStats?.username === entry.username && (
                        <span className="text-[10px] px-1 bg-primary/20 text-primary rounded">我</span>
                      )}
                    </div>
                  </td>
                  <td className="text-right py-2 px-2 font-mono-data">{formatMoney(entry.current_capital)}</td>
                  <td className="text-right py-2 px-2">
                    <span className={`flex items-center justify-end gap-1 ${entry.return_rate >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                      {entry.return_rate >= 0 ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />}
                      {formatPercent(entry.return_rate)}
                    </span>
                  </td>
                  <td className="text-right py-2 px-2 text-muted-foreground">{entry.total_trades}</td>
                  <td className="text-right py-2 px-2">
                    <span className={`${entry.win_rate >= 50 ? 'text-green-500' : 'text-red-500'}`}>
                      {entry.win_rate.toFixed(1)}%
                    </span>
                  </td>
                  <td className="text-right py-2 px-2 text-muted-foreground">{entry.positions_count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground py-12">
            <Target className="w-12 h-12 mb-3 opacity-30" />
            <p className="text-sm">暂无排行榜数据</p>
            <p className="text-xs mt-1">开始模拟交易后自动上榜</p>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
