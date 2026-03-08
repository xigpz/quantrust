/**
 * VisualStrategyEditor - 可视化策略编辑器
 * 拖拽式创建量化策略，无需编写代码
 */
import { useState, useCallback } from 'react';
import { 
  Database, 
  FunctionSquare, 
  GitBranch, 
  Filter, 
  Wallet, 
  Play, 
  Save, 
  Trash2, 
  Plus,
  Settings,
  Move,
  Link2,
  X,
  RefreshCw,
  AlertCircle
} from 'lucide-react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { toast } from 'sonner';

// 节点类型
type NodeType = 'data' | 'factor' | 'signal' | 'filter' | 'position';

// 节点定义
interface Node {
  id: string;
  type: NodeType;
  name: string;
  params: Record<string, any>;
  position: { x: number; y: number };
}

// 连接线定义
interface Connection {
  id: string;
  from: string;
  to: string;
}

// 节点类型配置
const NODE_TYPES: Record<NodeType, { icon: React.ReactNode; label: string; color: string; description: string }> = {
  data: { 
    icon: <Database className="w-4 h-4" />, 
    label: '数据源', 
    color: 'bg-blue-500',
    description: '获取股票数据'
  },
  factor: { 
    icon: <FunctionSquare className="w-4 h-4" />, 
    label: '因子', 
    color: 'bg-purple-500',
    description: '计算技术因子'
  },
  signal: { 
    icon: <GitBranch className="w-4 h-4" />, 
    label: '信号', 
    color: 'bg-green-500',
    description: '生成买卖信号'
  },
  filter: { 
    icon: <Filter className="w-4 h-4" />, 
    label: '过滤', 
    color: 'bg-orange-500',
    description: '过滤不符合条件的股票'
  },
  position: { 
    icon: <Wallet className="w-4 h-4" />, 
    label: '仓位', 
    color: 'bg-yellow-500',
    description: '计算持仓权重'
  },
};

// 内置节点模板
const NODE_TEMPLATES: Record<NodeType, { name: string; params: Record<string, any> }[]> = {
  data: [
    { name: 'A股全市场', params: { market: 'all' } },
    { name: '自选股', params: { source: 'watchlist' } },
    { name: '板块成分', params: { sector: '新能源' } },
  ],
  factor: [
    { name: 'MA5 均线', params: { period: 5, field: 'close' } },
    { name: 'MA20 均线', params: { period: 20, field: 'close' } },
    { name: 'RSI 指标', params: { period: 14 } },
    { name: 'MACD', params: { fast: 12, slow: 26, signal: 9 } },
    { name: '布林带', params: { period: 20, std: 2 } },
    { name: '成交量均值', params: { period: 20 } },
  ],
  signal: [
    { name: '均线金叉', params: { fast: 5, slow: 20 } },
    { name: '均线死叉', params: { fast: 5, slow: 20 } },
    { name: 'RSI超买', params: { threshold: 70 } },
    { name: 'RSI超卖', params: { threshold: 30 } },
    { name: '突破新高', params: { period: 20 } },
  ],
  filter: [
    { name: '市值过滤', params: { min: 10e8, max: 100e8 } },
    { name: '成交量过滤', params: { min: 1e7 } },
    { name: '涨跌幅过滤', params: { max: 9 } },
    { name: 'ST股票过滤', params: { exclude_st: true } },
  ],
  position: [
    { name: '等权重', params: { method: 'equal' } },
    { name: '按市值权重', params: { method: 'market_cap' } },
    { name: '按收益权重', params: { method: 'return' } },
  ],
};

function generateId(): string {
  return Math.random().toString(36).substr(2, 9);
}

export default function VisualStrategyEditor() {
  const [nodes, setNodes] = useState<Node[]>([]);
  const [connections, setConnections] = useState<Connection[]>([]);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [draggingType, setDraggingType] = useState<NodeType | null>(null);
  const [draggingNode, setDraggingNode] = useState<string | null>(null);
  const [canvasOffset, setCanvasOffset] = useState({ x: 0, y: 0 });

  // 添加节点
  const addNode = useCallback((type: NodeType, template: { name: string; params: Record<string, any> }, x?: number, y?: number) => {
    const newNode: Node = {
      id: generateId(),
      type,
      name: template.name,
      params: { ...template.params },
      position: { 
        x: x ?? 100 + Math.random() * 200, 
        y: y ?? 100 + Math.random() * 200 
      },
    };
    setNodes(prev => [...prev, newNode]);
    return newNode;
  }, []);

  // 删除节点
  const deleteNode = useCallback((id: string) => {
    setNodes(prev => prev.filter(n => n.id !== id));
    setConnections(prev => prev.filter(c => c.from !== id && c.to !== id));
    if (selectedNode?.id === id) setSelectedNode(null);
  }, [selectedNode]);

  // 更新节点参数
  const updateNodeParams = useCallback((id: string, params: Record<string, any>) => {
    setNodes(prev => prev.map(n => 
      n.id === id ? { ...n, params: { ...n.params, ...params } } : n
    ));
    if (selectedNode?.id === id) {
      setSelectedNode(prev => prev ? { ...prev, params: { ...prev.params, ...params } } : null);
    }
  }, [selectedNode]);

  // 处理拖拽开始
  const handleDragStart = (e: React.DragEvent, type: NodeType) => {
    setDraggingType(type);
    e.dataTransfer.effectAllowed = 'copy';
  };

  // 处理拖拽放置
  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    if (!draggingType) return;
    
    const rect = (e.target as HTMLElement).closest('.canvas-area')?.getBoundingClientRect();
    if (!rect) return;
    
    const x = e.clientX - rect.left - canvasOffset.x;
    const y = e.clientY - rect.top - canvasOffset.y;
    
    // 默认添加第一个模板
    const template = NODE_TEMPLATES[draggingType][0];
    addNode(draggingType, template, x, y);
    
    setDraggingType(null);
  };

  // 处理节点拖拽
  const handleNodeDragStart = (e: React.MouseEvent, nodeId: string) => {
    e.stopPropagation();
    setDraggingNode(nodeId);
    setSelectedNode(nodes.find(n => n.id === nodeId) || null);
  };

  // 处理画布鼠标移动（移动节点）
  const handleCanvasMouseMove = (e: React.MouseEvent) => {
    if (!draggingNode) return;
    
    const rect = (e.target as HTMLElement).closest('.canvas-area')?.getBoundingClientRect();
    if (!rect) return;
    
    const x = e.clientX - rect.left - canvasOffset.x;
    const y = e.clientY - rect.top - canvasOffset.y;
    
    setNodes(prev => prev.map(n => 
      n.id === draggingNode ? { ...n, position: { x, y } } : n
    ));
  };

  const handleCanvasMouseUp = () => {
    setDraggingNode(null);
  };

  // 生成策略代码预览
  const generateCode = () => {
    let code = '# 可视化策略\n\n';
    code += '# 节点配置:\n';
    nodes.forEach(node => {
      code += `# ${node.name} (${NODE_TYPES[node.type].label})\n`;
      code += `# params: ${JSON.stringify(node.params)}\n\n`;
    });
    code += '\n# 策略逻辑:\n';
    code += 'def handle_bar(context, data):\n';
    code += '    signals = []\n';
    
    const dataNodes = nodes.filter(n => n.type === 'data');
    const factorNodes = nodes.filter(n => n.type === 'factor');
    const signalNodes = nodes.filter(n => n.type === 'signal');
    const filterNodes = nodes.filter(n => n.type === 'filter');
    const positionNodes = nodes.filter(n => n.type === 'position');
    
    if (dataNodes.length) {
      code += `    # 数据源: ${dataNodes.map(n => n.name).join(', ')}\n`;
    }
    if (factorNodes.length) {
      code += `    # 因子: ${factorNodes.map(n => n.name).join(', ')}\n`;
    }
    if (signalNodes.length) {
      code += `    # 信号: ${signalNodes.map(n => n.name).join(', ')}\n`;
    }
    if (filterNodes.length) {
      code += `    # 过滤: ${filterNodes.map(n => n.name).join(', ')}\n`;
    }
    if (positionNodes.length) {
      code += `    # 仓位: ${positionNodes.map(n => n.name).join(', ')}\n`;
    }
    
    return code;
  };

  const [previewCode, setPreviewCode] = useState('');

  // 渲染节点
  const renderNode = (node: Node) => {
    const config = NODE_TYPES[node.type];
    const isSelected = selectedNode?.id === node.id;
    
    return (
      <div
        key={node.id}
        className={`absolute cursor-move select-none ${isSelected ? 'ring-2 ring-primary' : ''}`}
        style={{ left: node.position.x, top: node.position.y }}
        onMouseDown={(e) => handleNodeDragStart(e, node.id)}
      >
        <div className={`${config.color} text-white text-xs px-2 py-1 rounded-t-md flex items-center gap-1`}>
          {config.icon}
          <span>{config.label}</span>
        </div>
        <div className="bg-card border border-t-0 border-border rounded-b-md p-2 min-w-[120px]">
          <div className="font-medium text-sm">{node.name}</div>
          <div className="text-[10px] text-muted-foreground mt-1">
            {Object.entries(node.params).slice(0, 2).map(([k, v]) => (
              <div key={k}>{k}: {String(v)}</div>
            ))}
          </div>
        </div>
        {/* 删除按钮 */}
        <button
          onClick={(e) => { e.stopPropagation(); deleteNode(node.id); }}
          className="absolute -top-2 -right-2 w-5 h-5 bg-destructive text-destructive-foreground rounded-full flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity"
        >
          <X className="w-3 h-3" />
        </button>
      </div>
    );
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border">
        <div className="flex items-center gap-2">
          <GitBranch className="w-4 h-4 text-primary" />
          <h2 className="text-sm font-semibold">可视化策略编辑器</h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => { setPreviewCode(generateCode()); toast.success('策略已生成'); }}
            className="flex items-center gap-1 px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90"
          >
            <Play className="w-3 h-3" /> 生成
          </button>
          <button
            onClick={() => toast.info('保存功能开发中')}
            className="flex items-center gap-1 px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded hover:bg-secondary/80"
          >
            <Save className="w-3 h-3" /> 保存
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* 左侧：节点库 */}
        <div className="w-48 border-r border-border bg-card/50 flex flex-col">
          <div className="px-3 py-2 text-xs font-medium text-muted-foreground border-b border-border">
            节点组件库
          </div>
          <ScrollArea className="flex-1 p-2">
            {(Object.keys(NODE_TYPES) as NodeType[]).map(type => (
              <div key={type} className="mb-3">
                <div className="flex items-center gap-1.5 mb-1.5">
                  <div className={`w-5 h-5 rounded ${NODE_TYPES[type].color} flex items-center justify-center text-white`}>
                    {NODE_TYPES[type].icon}
                  </div>
                  <span className="text-xs font-medium">{NODE_TYPES[type].label}</span>
                </div>
                <div className="space-y-1">
                  {NODE_TEMPLATES[type].map((template, idx) => (
                    <div
                      key={idx}
                      draggable
                      onDragStart={(e) => handleDragStart(e, type)}
                      className="px-2 py-1.5 text-xs bg-muted/50 hover:bg-muted rounded cursor-grab flex items-center gap-1"
                    >
                      <Plus className="w-3 h-3 text-muted-foreground" />
                      {template.name}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </ScrollArea>
        </div>

        {/* 中间：画布 */}
        <div 
          className="flex-1 canvas-area bg-grid relative overflow-hidden"
          style={{ 
            backgroundImage: 'radial-gradient(circle, var(--border) 1px, transparent 1px)',
            backgroundSize: '20px 20px'
          }}
          onDragOver={(e) => e.preventDefault()}
          onDrop={handleDrop}
          onMouseMove={handleCanvasMouseMove}
          onMouseUp={handleCanvasMouseUp}
          onMouseLeave={handleCanvasMouseUp}
        >
          {nodes.length === 0 ? (
            <div className="absolute inset-0 flex items-center justify-center text-muted-foreground">
              <div className="text-center">
                <AlertCircle className="w-8 h-8 mx-auto mb-2 opacity-30" />
                <p className="text-sm">从左侧拖拽节点到画布</p>
                <p className="text-xs mt-1">开始构建你的策略</p>
              </div>
            </div>
          ) : (
            nodes.map(renderNode)
          )}
          
          {/* 画布工具栏 */}
          <div className="absolute bottom-4 left-4 flex gap-2">
            <button
              onClick={() => setCanvasOffset(prev => ({ ...prev, x: prev.x - 50 }))}
              className="p-1.5 bg-card border border-border rounded hover:bg-muted"
            >
              <Move className="w-3 h-3" />
            </button>
            <button
              onClick={() => { setNodes([]); setConnections([]); toast.success('画布已清空'); }}
              className="p-1.5 bg-card border border-border rounded hover:bg-muted text-destructive"
            >
              <Trash2 className="w-3 h-3" />
            </button>
          </div>
        </div>

        {/* 右侧：属性面板 */}
        <div className="w-56 border-l border-border bg-card/50 flex flex-col">
          <div className="px-3 py-2 text-xs font-medium text-muted-foreground border-b border-border">
            节点属性
          </div>
          {selectedNode ? (
            <ScrollArea className="flex-1 p-3">
              <div className="space-y-3">
                <div>
                  <label className="text-xs text-muted-foreground">类型</label>
                  <div className="flex items-center gap-1.5 mt-1">
                    <div className={`w-5 h-5 rounded ${NODE_TYPES[selectedNode.type].color} flex items-center justify-center text-white`}>
                      {NODE_TYPES[selectedNode.type].icon}
                    </div>
                    <span className="text-sm">{NODE_TYPES[selectedNode.type].label}</span>
                  </div>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">名称</label>
                  <input
                    type="text"
                    value={selectedNode.name}
                    onChange={(e) => updateNodeParams(selectedNode.id, { name: e.target.value })}
                    className="w-full mt-1 px-2 py-1 text-sm bg-background border border-border rounded focus:outline-none focus:ring-1 focus:ring-primary"
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">参数配置</label>
                  <div className="mt-1 space-y-2">
                    {Object.entries(selectedNode.params).map(([key, value]) => (
                      <div key={key}>
                        <label className="text-[10px] text-muted-foreground">{key}</label>
                        {typeof value === 'number' ? (
                          <input
                            type="number"
                            value={value}
                            onChange={(e) => updateNodeParams(selectedNode.id, { [key]: parseFloat(e.target.value) })}
                            className="w-full mt-0.5 px-2 py-1 text-sm bg-background border border-border rounded focus:outline-none focus:ring-1 focus:ring-primary"
                          />
                        ) : typeof value === 'boolean' ? (
                          <label className="flex items-center gap-2 mt-0.5">
                            <input
                              type="checkbox"
                              checked={value}
                              onChange={(e) => updateNodeParams(selectedNode.id, { [key]: e.target.checked })}
                              className="w-3 h-3"
                            />
                            <span className="text-xs">{value ? '是' : '否'}</span>
                          </label>
                        ) : (
                          <input
                            type="text"
                            value={String(value)}
                            onChange={(e) => updateNodeParams(selectedNode.id, { [key]: e.target.value })}
                            className="w-full mt-0.5 px-2 py-1 text-sm bg-background border border-border rounded focus:outline-none focus:ring-1 focus:ring-primary"
                          />
                        )}
                      </div>
                    ))}
                  </div>
                </div>
                <button
                  onClick={() => deleteNode(selectedNode.id)}
                  className="w-full py-1.5 text-xs text-destructive border border-destructive rounded hover:bg-destructive/10"
                >
                  删除节点
                </button>
              </div>
            </ScrollArea>
          ) : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground p-4">
              <p className="text-xs text-center">选择节点查看属性</p>
            </div>
          )}
        </div>
      </div>

      {/* 底部：代码预览 */}
      {previewCode && (
        <div className="h-32 border-t border-border bg-card/50 flex flex-col">
          <div className="flex items-center justify-between px-3 py-1.5 border-b border-border">
            <span className="text-xs font-medium">策略代码预览</span>
            <button onClick={() => setPreviewCode('')} className="text-muted-foreground hover:text-foreground">
              <X className="w-3 h-3" />
            </button>
          </div>
          <ScrollArea className="flex-1 p-3">
            <pre className="text-[10px] font-mono text-muted-foreground whitespace-pre-wrap">
              {previewCode}
            </pre>
          </ScrollArea>
        </div>
      )}
    </div>
  );
}
