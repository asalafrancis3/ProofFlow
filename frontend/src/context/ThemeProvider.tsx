import { createContext, useContext, useEffect, useMemo, type ReactNode } from 'react'
import {
  ThemeProvider as NextThemesProvider,
  useTheme as useNextTheme,
  type ThemeProviderProps,
} from 'next-themes'

export type ThemeName = 'light' | 'dark' | 'high-contrast' | 'system'
type ResolvedTheme = 'light' | 'dark' | 'high-contrast'

interface ThemeContextValue {
  theme: ThemeName
  resolvedTheme: ResolvedTheme
  isDark: boolean
  isHighContrast: boolean
  isReady: boolean
  setTheme: (theme: ThemeName) => void
  toggleTheme: () => void
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)

function ThemeContextBridge({ children }: { children: ReactNode }) {
  const { theme, resolvedTheme, setTheme } = useNextTheme()

  const currentTheme: ThemeName =
    theme === 'light' || theme === 'dark' || theme === 'system' || theme === 'high-contrast' ? theme : 'system'
  let currentResolvedTheme: ResolvedTheme = 'light'
  if (resolvedTheme === 'dark') currentResolvedTheme = 'dark'
  if (resolvedTheme === 'high-contrast' || theme === 'high-contrast') currentResolvedTheme = 'high-contrast'
  const isDark = currentResolvedTheme === 'dark'
  const isHighContrast = currentResolvedTheme === 'high-contrast'

  // Add theme-ready class after mount so transitions don't fire on initial paint
  useEffect(() => {
    const raf = requestAnimationFrame(() => {
      document.documentElement.classList.add('theme-ready')
    })
    return () => cancelAnimationFrame(raf)
  }, [])

  const value = useMemo<ThemeContextValue>(
    () => ({
      theme: currentTheme,
      resolvedTheme: currentResolvedTheme,
      isDark,
      isHighContrast,
      isReady: theme !== undefined || resolvedTheme !== undefined,
      setTheme: (nextTheme) => setTheme(nextTheme),
      toggleTheme: () => setTheme(isDark ? 'light' : isHighContrast ? 'light' : 'dark'),
    }),
    [currentResolvedTheme, currentTheme, isDark, isHighContrast, resolvedTheme, setTheme, theme]
  )

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

export function ThemeProvider({ children, ...props }: ThemeProviderProps) {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      storageKey="proofflow-theme"
      // transitions are handled via CSS + theme-ready class instead
      {...props}
    >
      <ThemeContextBridge>{children}</ThemeContextBridge>
    </NextThemesProvider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)

  if (!context) {
    throw new Error('UseTheme must be used within ThemeProvider.')
  }

  return context
}
