/**
 * Core Web Vitals and Performance Monitoring
 * 
 * Tracks and reports:
 * - Largest Contentful Paint (LCP)
 * - First Input Delay (FID) / Interaction to Next Paint (INP)
 * - Cumulative Layout Shift (CLS)
 * - First Contentful Paint (FCP)
 * - Time to First Byte (TTFB)
 */

interface WebVital {
  name: string
  value: number
  rating?: 'good' | 'needs-improvement' | 'poor'
  delta?: number
  id?: string
}

interface PerformanceMetrics {
  lcp?: WebVital
  fid?: WebVital
  inp?: WebVital
  cls?: WebVital
  fcp?: WebVital
  ttfb?: WebVital
  navigationTime?: number
  resourceTime?: number
}

const THRESHOLDS = {
  lcp: { good: 2500, poor: 4000 },
  fid: { good: 100, poor: 300 },
  inp: { good: 200, poor: 500 },
  cls: { good: 0.1, poor: 0.25 },
  fcp: { good: 1800, poor: 3000 },
  ttfb: { good: 600, poor: 1800 },
}

function getRating(
  value: number,
  metric: keyof typeof THRESHOLDS
): 'good' | 'needs-improvement' | 'poor' {
  const { good, poor } = THRESHOLDS[metric]
  if (value <= good) return 'good'
  if (value <= poor) return 'needs-improvement'
  return 'poor'
}

/**
 * Initialize Web Vitals monitoring
 */
export function initWebVitals(
  onMetric?: (metric: WebVital) => void,
  debug = false
): () => void {
  const metrics: PerformanceMetrics = {}

  // Use native Web Vitals if available
  if ('PerformanceObserver' in window) {
    // LCP
    try {
      const lcpObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries()
        const lastEntry = entries[entries.length - 1]
        const e = lastEntry as { renderTime?: number; loadTime?: number; id?: string }
        const lcp: WebVital = {
          name: 'LCP',
          value: e.renderTime ?? e.loadTime ?? 0,
          rating: getRating(e.renderTime ?? e.loadTime ?? 0, 'lcp'),
          id: e.id,
        }
        metrics.lcp = lcp
        onMetric?.(lcp)
      })
      lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] })
    } catch (e) {
      if (debug) console.error('LCP observer error:', e)
    }

    // CLS
    try {
      let clsValue = 0
      const clsObserver = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          const e = entry as { hadRecentInput?: boolean; value?: number }
          if (!('hadRecentInput' in e) || !e.hadRecentInput) {
            clsValue += e.value ?? 0
          }
        }
        const cls: WebVital = {
          name: 'CLS',
          value: clsValue,
          rating: getRating(clsValue, 'cls'),
        }
        metrics.cls = cls
        onMetric?.(cls)
      })
      clsObserver.observe({ entryTypes: ['layout-shift'] })
    } catch (e) {
      if (debug) console.error('CLS observer error:', e)
    }

    // FCP
    try {
      const fcpObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries()
        const fcpEntry = entries.find((e) => e.name === 'first-contentful-paint')
        if (fcpEntry) {
          const fcp: WebVital = {
            name: 'FCP',
            value: fcpEntry.startTime,
            rating: getRating(fcpEntry.startTime, 'fcp'),
          }
          metrics.fcp = fcp
          onMetric?.(fcp)
        }
      })
      fcpObserver.observe({ entryTypes: ['paint'] })
    } catch (e) {
      if (debug) console.error('FCP observer error:', e)
    }

    // INP (Interaction to Next Paint) - replacement for FID
    try {
      const inpObserver = new PerformanceObserver((list) => {
        const entries = list.getEntries()
        let maxINP = 0
        for (const entry of entries) {
          if ('processingDuration' in entry) {
            maxINP = Math.max(maxINP, entry.duration)
          }
        }
        if (maxINP > 0) {
          const inp: WebVital = {
            name: 'INP',
            value: maxINP,
            rating: getRating(maxINP, 'inp'),
          }
          metrics.inp = inp
          onMetric?.(inp)
        }
      })
      inpObserver.observe({ entryTypes: ['event'] })
    } catch (e) {
      if (debug) console.error('INP observer error:', e)
    }
  }

  // Navigation and timing metrics
  if ('PerformanceTiming' in window) {
    const timing = performance.timing
    const navigationTime = timing.loadEventEnd - timing.navigationStart
    const resourceTime = timing.responseEnd - timing.fetchStart
    const ttfb = timing.responseStart - timing.fetchStart

    metrics.navigationTime = navigationTime
    metrics.resourceTime = resourceTime

    if (ttfb > 0) {
      const ttfbMetric: WebVital = {
        name: 'TTFB',
        value: ttfb,
        rating: getRating(ttfb, 'ttfb'),
      }
      metrics.ttfb = ttfbMetric
      onMetric?.(ttfbMetric)
    }
  }

  // Return cleanup function
  return () => {
    // Performance observer cleanup handled automatically
  }
}

/**
 * Send metrics to analytics service
 */
export function sendMetricsToAnalytics(metrics: PerformanceMetrics): void {
  const metricsToSend = Object.entries(metrics)
    .map(([, metric]) => metric)
    .filter((m) => m !== undefined)

  if (metricsToSend.length === 0) return

  // Use sendBeacon to ensure delivery even if page unloads
  if (navigator.sendBeacon) {
    const data = JSON.stringify({
      event: 'web_vitals',
      metrics: metricsToSend,
      timestamp: new Date().toISOString(),
      url: window.location.href,
    })

    navigator.sendBeacon('/api/metrics', data)
  }
}

/**
 * Get all currently collected metrics
 */
export function getMetrics(): PerformanceMetrics {
  return {
    ...performance.timing,
  } as PerformanceMetrics
}

/**
 * Check if metrics meet performance budgets
 */
export function validatePerformanceBudgets(
  metrics: PerformanceMetrics
): { passed: boolean; violations: string[] } {
  const violations: string[] = []

  // LCP budget: 2.5s
  if (metrics.lcp && metrics.lcp.value > THRESHOLDS.lcp.good) {
    violations.push(
      `LCP exceeded budget (${metrics.lcp.value.toFixed(0)}ms > ${THRESHOLDS.lcp.good}ms)`
    )
  }

  // CLS budget: 0.1
  if (metrics.cls && metrics.cls.value > THRESHOLDS.cls.good) {
    violations.push(
      `CLS exceeded budget (${metrics.cls.value.toFixed(3)} > ${THRESHOLDS.cls.good})`
    )
  }

  // FCP budget: 1.8s
  if (metrics.fcp && metrics.fcp.value > THRESHOLDS.fcp.good) {
    violations.push(
      `FCP exceeded budget (${metrics.fcp.value.toFixed(0)}ms > ${THRESHOLDS.fcp.good}ms)`
    )
  }

  // TTFB budget: 600ms
  if (metrics.ttfb && metrics.ttfb.value > THRESHOLDS.ttfb.good) {
    violations.push(
      `TTFB exceeded budget (${metrics.ttfb.value.toFixed(0)}ms > ${THRESHOLDS.ttfb.good}ms)`
    )
  }

  return {
    passed: violations.length === 0,
    violations,
  }
}

export type { WebVital, PerformanceMetrics }
