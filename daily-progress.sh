#!/bin/bash
# QuantRust 每日推进脚本
# 任务：检查项目、代码优化、研究改进、提交进度

REPO_DIR="/root/.openclaw/workspace/quantrust"
cd $REPO_DIR

# 获取今天的日期
TODAY=$(date '+%Y-%m-%d')

# 检查是否已经完成今日任务
if grep -q "## $TODAY" PROGRESS.md 2>/dev/null; then
    echo "今日任务已记录"
    exit 0
fi

echo "开始每日推进..."

# 1. 检查项目状态
echo "检查项目状态..."

# 2. 查看最近的提交
echo "最近提交:"
git log --oneline -3

# 3. 添加进度记录
cat >> PROGRESS.md << EOF

---

## $TODAY
**状态**: 
**发现**:
**下一步**:
EOF

# 4. 提交
git add -A
git commit -m "Daily progress: $TODAY" || echo "没有新更改"
git push origin main

echo "每日推进完成: $TODAY"
