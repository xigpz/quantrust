import { useState } from 'react';
import { sendAIChat, type ChatMessage, type AIChatResponse } from '@/hooks/useMarketData';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Loader2, Send, Bot, User } from 'lucide-react';

export default function AIChatPanel() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [symbol, setSymbol] = useState('');
  const [currentResponse, setCurrentResponse] = useState<string | null>(null);

  const handleSend = async () => {
    if (!input.trim()) return;
    const userMsg: ChatMessage = { role: 'user', content: input };
    setMessages(prev => [...prev, userMsg]);
    setCurrentResponse(null);
    setLoading(true);
    try {
      const res = await sendAIChat({
        message: input,
        symbol: symbol || undefined,
        context: messages
      });
      if (res.success) {
        const aiMsg: ChatMessage = { role: 'assistant', content: res.data.response };
        setMessages(prev => [...prev, aiMsg]);
        setCurrentResponse(res.data.response);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
      setInput('');
    }
  };

  const handleSuggestion = (suggestion: string) => {
    setInput(suggestion);
  };

  const handleClear = () => {
    setMessages([]);
    setCurrentResponse(null);
  };

  return (
    <div className="p-4 space-y-4 overflow-auto h-full flex flex-col">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Bot className="h-6 w-6" />
          AI 量化助手
        </h2>
        <Button variant="outline" size="sm" onClick={handleClear}>
          清空对话
        </Button>
      </div>

      {/* 股票代码输入 */}
      <Card>
        <CardContent className="pt-4">
          <div className="flex gap-2">
            <Input
              placeholder="股票代码（可选）"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value)}
              className="w-40"
            />
            <span className="text-sm text-muted-foreground self-center">
              可指定股票进行针对性分析
            </span>
          </div>
        </CardContent>
      </Card>

      {/* 聊天记录 */}
      <Card className="flex-1 min-h-[300px] overflow-hidden">
        <CardHeader className="pb-2">
          <CardTitle className="text-lg">对话记录</CardTitle>
        </CardHeader>
        <CardContent className="overflow-y-auto h-[400px] space-y-4">
          {messages.length === 0 && !currentResponse && (
            <div className="text-center text-muted-foreground py-8">
              <Bot className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p>您好！我是您的量化投资助手</p>
              <p className="text-sm mt-2">可以问我关于股票分析、交易策略等问题</p>
            </div>
          )}
          {messages.map((msg, i) => (
            <div key={i} className={`flex gap-2 ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
              {msg.role === 'assistant' && <Bot className="h-5 w-5 mt-1 text-blue-500" />}
              <div className={`max-w-[80%] p-3 rounded-lg ${
                msg.role === 'user' ? 'bg-blue-500 text-white' : 'bg-muted'
              }`}>
                <div className="whitespace-pre-wrap text-sm">{msg.content}</div>
              </div>
              {msg.role === 'user' && <User className="h-5 w-5 mt-1 text-green-500" />}
            </div>
          ))}
          {loading && (
            <div className="flex gap-2">
              <Bot className="h-5 w-5 mt-1 text-blue-500" />
              <div className="bg-muted p-3 rounded-lg">
                <Loader2 className="h-4 w-4 animate-spin" />
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 建议问题 */}
      {!loading && messages.length > 0 && currentResponse && (
        <Card>
          <CardContent className="pt-4">
            <p className="text-sm text-muted-foreground mb-2">您可能想了解：</p>
            <div className="flex flex-wrap gap-2">
              {['推荐一支潜力股', '如何设置止损', '当前市场趋势如何', '我的持仓应该卖出吗'].map((s, i) => (
                <Button
                  key={i}
                  variant="outline"
                  size="sm"
                  onClick={() => handleSuggestion(s)}
                >
                  {s}
                </Button>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* 输入框 */}
      <Card>
        <CardContent className="pt-4">
          <div className="flex gap-2">
            <Input
              placeholder="输入您的问题..."
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSend()}
              disabled={loading}
            />
            <Button onClick={handleSend} disabled={loading || !input.trim()}>
              {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
