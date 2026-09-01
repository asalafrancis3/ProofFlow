/**
 * Currency and unit localization utilities (Issue #779)
 */

export type WeightUnit = 'kg' | 'lb'
export type DistanceUnit = 'km' | 'mi'

export interface LocaleConfig {
  currency: string
  currencyLocale: string
  weightUnit: WeightUnit
  distanceUnit: DistanceUnit
}

// ── Locale database ───────────────────────────────────────────────────────────

const LOCALE_DB: Record<string, LocaleConfig> = {
  en: { currency: 'USD', currencyLocale: 'en-US', weightUnit: 'lb', distanceUnit: 'mi' },
  'en-US': { currency: 'USD', currencyLocale: 'en-US', weightUnit: 'lb', distanceUnit: 'mi' },
  'en-GB': { currency: 'GBP', currencyLocale: 'en-GB', weightUnit: 'kg', distanceUnit: 'km' },
  'en-AU': { currency: 'AUD', currencyLocale: 'en-AU', weightUnit: 'kg', distanceUnit: 'km' },
  es: { currency: 'EUR', currencyLocale: 'es-ES', weightUnit: 'kg', distanceUnit: 'km' },
  'es-MX': { currency: 'MXN', currencyLocale: 'es-MX', weightUnit: 'kg', distanceUnit: 'km' },
  fr: { currency: 'EUR', currencyLocale: 'fr-FR', weightUnit: 'kg', distanceUnit: 'km' },
  'fr-CA': { currency: 'CAD', currencyLocale: 'fr-CA', weightUnit: 'kg', distanceUnit: 'km' },
  zh: { currency: 'CNY', currencyLocale: 'zh-CN', weightUnit: 'kg', distanceUnit: 'km' },
  ar: { currency: 'SAR', currencyLocale: 'ar-SA', weightUnit: 'kg', distanceUnit: 'km' },
  de: { currency: 'EUR', currencyLocale: 'de-DE', weightUnit: 'kg', distanceUnit: 'km' },
  pt: { currency: 'EUR', currencyLocale: 'pt-PT', weightUnit: 'kg', distanceUnit: 'km' },
  'pt-BR': { currency: 'BRL', currencyLocale: 'pt-BR', weightUnit: 'kg', distanceUnit: 'km' },
}

const DEFAULT_CONFIG: LocaleConfig = {
  currency: 'USD',
  currencyLocale: 'en-US',
  weightUnit: 'kg',
  distanceUnit: 'km',
}

const OVERRIDE_KEY = 'scavngr_locale_override'

// ── Locale detection ──────────────────────────────────────────────────────────

export function detectLocale(): string {
  return localStorage.getItem(OVERRIDE_KEY) ?? navigator.language ?? 'en'
}

export function getLocaleConfig(locale: string): LocaleConfig {
  return LOCALE_DB[locale] ?? LOCALE_DB[locale.split('-')[0]] ?? DEFAULT_CONFIG
}

export function setLocaleOverride(locale: string): void {
  localStorage.setItem(OVERRIDE_KEY, locale)
}

export function clearLocaleOverride(): void {
  localStorage.removeItem(OVERRIDE_KEY)
}

// Formatters moved to lib/format.ts
import { formatNumber } from './format'
export { formatCurrency } from './format'

const KG_TO_LB = 2.20462
const KM_TO_MI = 0.621371

export function formatWeight(kg: number, locale?: string): string {
  const resolved = locale ?? detectLocale()
  const config = getLocaleConfig(resolved)
  const value = config.weightUnit === 'lb' ? kg * KG_TO_LB : kg
  return `${formatNumber(value, resolved, 1)} ${config.weightUnit}`
}

export function formatDistance(km: number, locale?: string): string {
  const resolved = locale ?? detectLocale()
  const config = getLocaleConfig(resolved)
  const value = config.distanceUnit === 'mi' ? km * KM_TO_MI : km
  return `${formatNumber(value, resolved, 1)} ${config.distanceUnit}`
}

export function formatDate(date: Date | number | string, locale?: string): string {
  const resolved = locale ?? detectLocale()
  const config = getLocaleConfig(resolved)
  return new Intl.DateTimeFormat(config.currencyLocale, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(new Date(date))
}
