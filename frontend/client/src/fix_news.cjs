const fs = require('fs');
let content = fs.readFileSync('MarketCommandCenterPanel.tsx', 'utf8');

// Find the news section with the ternary and fix the missing closing paren
const oldNews = `                  </div>
                ) : (
                  <div className="text-center py-8 text-muted-foreground text-sm">
                    暂无新闻数据
                  </div>
                )}`;

const newNews = `                  </div>
                )) : (
                  <div className="text-center py-8 text-muted-foreground text-sm">
                    暂无新闻数据
                  </div>
                )}`;

if (content.includes(oldNews)) {
  content = content.replace(oldNews, newNews);
  console.log('Fixed: added extra closing paren');
} else {
  console.log('Pattern not found');
}

fs.writeFileSync('MarketCommandCenterPanel.tsx', content);
