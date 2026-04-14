import { useState, useEffect } from 'react';

interface Strategy {
  id: string;
  name: string;
  description: string;
  code: string;
  language: string;
  created_at: string;
  updated_at: string;
}

const DEFAULT_CODE = `# QuantRust 策略模板
# 返回 signals: [{symbol, action, price, quantity}]

def init(context):
    context.symbols = ['000001', '600519']
    context.buy_threshold = 0.02
    context.sell_threshold = 0.05

def handle_bar(context, data):
    signals = []
    for symbol in context.symbols:
        prices = data.history(symbol, ['close', 'volume'], 20, '1d')
        if len(prices) < 20:
            continue
            
        # 简单均线策略
        ma5 = prices['close'].iloc[-5:].mean()
        ma20 = prices['close'].mean()
        
        if ma5 > ma20 * (1 + context.buy_threshold):
            signals.append({
                'symbol': symbol,
                'action': 'buy',
                'price': prices['close'].iloc[-1],
                'quantity': 100
            })
        elif ma5 < ma20 * (1 - context.sell_threshold):
            signals.append({
                'symbol': symbol,
                'action': 'sell',
                'price': prices['close'].iloc[-1],
                'quantity': 100
            })
    
    return signals
`;

export default function StrategyIDE() {
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [currentStrategy, setCurrentStrategy] = useState<Strategy | null>(null);
  const [code, setCode] = useState(DEFAULT_CODE);
  const [name, setName] = useState('新策略');
  const [description, setDescription] = useState('');
  const [symbol, setSymbol] = useState('600519.SH');
  const [capital, setCapital] = useState(100000);
  const [running, setRunning] = useState(false);
  const [output, setOutput] = useState('');

  useEffect(() => {
    const raw = localStorage.getItem('quantrust_strategy_ide_draft');
    if (!raw) return;
    try {
      const draft = JSON.parse(raw) as {
        name?: string;
        description?: string;
        code?: string;
      };
      if (draft.name) setName(draft.name);
      if (typeof draft.description === 'string') setDescription(draft.description);
      if (draft.code) setCode(draft.code);
      setOutput('已从模板市场加载策略代码，可直接保存或运行回测。');
      localStorage.removeItem('quantrust_strategy_ide_draft');
    } catch {
      localStorage.removeItem('quantrust_strategy_ide_draft');
    }
  }, []);

  // 加载策略列表
  useEffect(() => {
    fetch('/api/strategies')
      .then(r => r.json())
      .then(data => {
        if (data.data) setStrategies(data.data);
      })
      .catch(console.error);
  }, []);

  // 保存策略
  const saveStrategy = async () => {
    const res = await fetch('/api/strategies', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, description, code, language: 'python' })
    });
    const data = await res.json();
    if (data.success) {
      setOutput('策略保存成功！');
      // 刷新列表
      fetch('/api/strategies')
        .then(r => r.json())
        .then(data => {
          if (data.data) setStrategies(data.data);
        });
    } else {
      setOutput('保存失败: ' + data.message);
    }
  };

  // 运行回测
  const runBacktest = async () => {
    if (!symbol.trim()) {
      setOutput('回测失败: 请输入有效标的代码');
      return;
    }
    if (!capital || capital <= 0) {
      setOutput('回测失败: 初始资金必须大于 0');
      return;
    }
    setRunning(true);
    setOutput('正在运行回测...');

    try {
      const res = await fetch('/api/backtest/code', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          code,
          symbol: symbol.trim(),
          initial_capital: capital,
          period: '1d',
          count: 500,
        })
      });

      const data = await res.json();
      if (data.success) {
        setOutput(JSON.stringify(data.data, null, 2));
      } else {
        setOutput('回测失败: ' + data.message);
      }
    } catch (e) {
      setOutput(`回测失败: ${e instanceof Error ? e.message : '网络异常'}`);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="flex h-full gap-4 p-4">
      {/* 左侧：策略列表 */}
      <div className="w-64 bg-gray-800 rounded-lg p-4">
        <h3 className="text-lg font-bold mb-4">我的策略</h3>
        <button
          onClick={() => { setCurrentStrategy(null); setCode(DEFAULT_CODE); setName('新策略'); }}
          className="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded mb-4"
        >
          + 新建策略
        </button>
        <div className="space-y-2 overflow-auto">
          {strategies.map(s => (
            <div
              key={s.id}
              onClick={() => { setCurrentStrategy(s); setCode(s.code); setName(s.name); setDescription(s.description || ''); }}
              className={`p-2 rounded cursor-pointer ${currentStrategy?.id === s.id ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'}`}
            >
              <div className="font-medium">{s.name}</div>
              <div className="text-xs text-gray-400">{s.language}</div>
            </div>
          ))}
        </div>
      </div>

      {/* 中间：代码编辑器 */}
      <div className="flex-1 flex flex-col bg-gray-900 rounded-lg p-4">
        <div className="flex gap-4 mb-4">
          <input
            type="text"
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder="策略名称"
            className="flex-1 bg-gray-800 text-white px-4 py-2 rounded"
          />
          <input
            type="text"
            value={description}
            onChange={e => setDescription(e.target.value)}
            placeholder="策略描述"
            className="flex-1 bg-gray-800 text-white px-4 py-2 rounded"
          />
          <input
            type="text"
            value={symbol}
            onChange={e => setSymbol(e.target.value)}
            placeholder="回测标的，如 600519.SH"
            className="w-52 bg-gray-800 text-white px-4 py-2 rounded"
          />
          <input
            type="number"
            value={capital}
            onChange={e => setCapital(Number(e.target.value) || 0)}
            placeholder="初始资金"
            className="w-40 bg-gray-800 text-white px-4 py-2 rounded"
          />
        </div>
        
        <textarea
          value={code}
          onChange={e => setCode(e.target.value)}
          className="flex-1 bg-gray-800 text-green-400 font-mono p-4 rounded resize-none"
          spellCheck={false}
        />

        <div className="flex gap-4 mt-4">
          <button
            onClick={saveStrategy}
            className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded"
          >
            保存
          </button>
          <button
            onClick={runBacktest}
            disabled={running}
            className="bg-green-600 hover:bg-green-700 text-white px-6 py-2 rounded disabled:opacity-50"
          >
            {running ? '运行中...' : '运行回测'}
          </button>
        </div>
      </div>

      {/* 右侧：输出 */}
      <div className="w-80 bg-gray-800 rounded-lg p-4">
        <h3 className="text-lg font-bold mb-4">输出</h3>
        <pre className="text-sm text-gray-300 whitespace-pre-wrap overflow-auto h-[calc(100%-2rem)]">
          {output || '点击"运行回测"查看结果...'}
        </pre>
      </div>
    </div>
  );
}
