/**
 * Performance Monitoring - 前端性能监控
 */

// 性能指标存储
interface PerformanceMetrics {
  pageLoad: number;
  firstContentfulPaint: number;
  largestContentfulPaint: number;
  timeToInteractive: number;
  cumulativeLayoutShift: number;
  firstInputDelay: number;
}

class PerformanceMonitor {
  private metrics: Partial<PerformanceMetrics> = {};
  private initialized = false;

  constructor() {
    this.init();
  }

  private init() {
    if (this.initialized) return;
    this.initialized = true;

    // 页面加载时间
    window.addEventListener('load', () => {
      const perfData = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      if (perfData) {
        this.metrics.pageLoad = perfData.loadEventEnd - perfData.fetchStart;
      }
    });

    // First Contentful Paint
    const fcpObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const fcp = entries[entries.length - 1] as PerformancePaintTiming;
      this.metrics.firstContentfulPaint = fcp.startTime;
    });
    try {
      fcpObserver.observe({ type: 'paint', buffered: true });
    } catch (e) {
      // FCP not supported
    }

    // Largest Contentful Paint
    const lcpObserver = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const lcp = entries[entries.length - 1] as PerformancePaintTiming;
      this.metrics.largestContentfulPaint = lcp.startTime;
    });
    try {
      lcpObserver.observe({ type: 'largest-contentful-paint', buffered: true });
    } catch (e) {
      // LCP not supported
    }

    // Cumulative Layout Shift
    const clsObserver = new PerformanceObserver((list) => {
      let cls = 0;
      for (const entry of list.getEntries()) {
        const layoutShift = entry as PerformanceEntry & { value: number };
        if (!layoutShift.hadRecentInput) {
          cls += layoutShift.value;
        }
      }
      this.metrics.cumulativeLayoutShift = cls;
    });
    try {
      clsObserver.observe({ type: 'layout-shift', buffered: true });
    } catch (e) {
      // CLS not supported
    }

    // First Input Delay
    const fidObserver = new PerformanceObserver((list) => {
      const firstInput = list.getEntries()[0] as PerformanceEventTiming;
      this.metrics.firstInputDelay = firstInput.processingStart - firstInput.startTime;
    });
    try {
      fidObserver.observe({ type: 'first-input', buffered: true });
    } catch (e) {
      // FID not supported
    }

    // 定期上报性能数据
    setTimeout(() => this.reportMetrics(), 5000);
  }

  private reportMetrics() {
    const metrics = this.getMetrics();
    // 开发环境输出到 console
    if (import.meta.env.DEV) {
      console.log('[Performance]', metrics);
    }
    // 可以发送到监控服务
    // fetch('/api/metrics', { method: 'POST', body: JSON.stringify(metrics) });
  }

  getMetrics(): Partial<PerformanceMetrics> {
    return { ...this.metrics };
  }
}

// 导出单例
export const perfMonitor = new PerformanceMonitor();

// 路由变化监控
export function initRouteMetrics() {
  if (typeof window === 'undefined') return;

  let lastPath = window.location.pathname;

  // 监听 popstate
  window.addEventListener('popstate', () => {
    const currentPath = window.location.pathname;
    if (currentPath !== lastPath) {
      lastPath = currentPath;
      // 路由切换时可以记录页面切换时间
      console.log(`[Route] Navigated to ${currentPath}`);
    }
  });

  // 重写 pushState 和 replaceState
  const originalPushState = window.history.pushState;
  const originalReplaceState = window.history.replaceState;

  window.history.pushState = function (...args) {
    originalPushState.apply(window.history, args);
    const currentPath = window.location.pathname;
    if (currentPath !== lastPath) {
      lastPath = currentPath;
      console.log(`[Route] Navigated to ${currentPath}`);
    }
  };

  window.history.replaceState = function (...args) {
    originalReplaceState.apply(window.history, args);
  };
}
