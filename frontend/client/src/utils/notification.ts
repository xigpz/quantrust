// 浏览器通知工具
export class BrowserNotification {
  static permission: NotificationPermission = 'default';
  
  // 请求通知权限
  static async request(): Promise<boolean> {
    if (!('Notification' in window)) {
      console.warn('浏览器不支持通知');
      return false;
    }
    
    if (Notification.permission === 'granted') {
      return true;
    }
    
    if (Notification.permission !== 'denied') {
      const permission = await Notification.requestPermission();
      this.permission = permission;
      return permission === 'granted';
    }
    
    return false;
  }
  
  // 发送通知
  static notify(title: string, options?: NotificationOptions): void {
    if (Notification.permission === 'granted') {
      new Notification(title, {
        icon: '/favicon.ico',
        badge: '/favicon.ico',
        ...options,
      });
    }
  }
  
  // 异动提醒
  static notifyAnomaly(stocks: { symbol: string; name: string; type: string }[]): void {
    if (stocks.length === 0) return;
    
    const title = stocks.length === 1 
      ? `异动提醒: ${stocks[0].symbol}`
      : `异动提醒: ${stocks.length}只股票`;
    
    const body = stocks.slice(0, 3).map(s => `${s.name} - ${s.type}`).join('\n');
    
    this.notify(title, {
      body,
      tag: 'anomaly-alert',
      requireInteraction: true,
    });
  }
  
  // 涨停提醒
  static notifyLimitUp(symbol: string, name: string): void {
    this.notify('🚀 涨停提醒', {
      body: `${name} (${symbol}) 已涨停！`,
      tag: 'limit-up',
    });
  }
  
  // 风险提醒
  static notifyRisk(symbol: string, name: string, reason: string): void {
    this.notify('⚠️ 风险提醒', {
      body: `${name}: ${reason}`,
      tag: 'risk-alert',
    });
  }
}

export default BrowserNotification;
