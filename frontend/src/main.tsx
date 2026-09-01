import React from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider, MutationCache, QueryCache } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { persistQueryClient } from '@tanstack/react-query-persist-client'
import { Toaster, toast } from 'sonner'
import { App } from '@/App'
import { AuthProvider } from '@/context/AuthContext'
import { WalletProvider } from '@/context/WalletContext'
import { ContractProvider } from '@/context/ContractContext'
import { ThemeProvider, useTheme } from '@/context/ThemeProvider'
import { StoreProvider } from '@/store'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { getErrorMessage } from '@/lib/contractErrors'
import { initWebVitals } from '@/lib/webVitals'
import { getDB, setQueryData } from '@/lib/indexedDB'
import '@/i18n/config'
import './index.css'

initWebVitals((metric) => {
  if (!import.meta.env.DEV) {
    try {
      navigator.sendBeacon('/api/metrics', JSON.stringify({
        metric: metric.name,
        value: metric.value,
        rating: metric.rating,
        timestamp: new Date().toISOString(),
      }))
    } catch {
      // Silently fail in production
    }
  }
})

const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error) => toast.error(getErrorMessage(error))
  }),
  mutationCache: new MutationCache({
    onError: (error) => toast.error(getErrorMessage(error))
  }),
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 5 * 60 * 1000,
      gcTime: 24 * 60 * 60 * 1000,
    }
  }
})

persistQueryClient({
  queryClient,
  persister: {
    persistClient: async (persistedClient) => {
      await getDB()
      const queries = persistedClient.clientState.queries
      for (const query of queries) {
        await setQueryData(
          query.queryKey.join('/'),
          query.state.data,
          query.queryKey as string[]
        )
      }
    },
    restoreClient: async () => {
      const db = await getDB()
      const tx = db.transaction('queries', 'readonly')
      const store = tx.objectStore('queries')
      const queries: Record<string, unknown> = {}
      for await (const cursor of store) {
        queries[cursor.key as string] = cursor.value
      }
      await tx.done
      return undefined
    },
    removeClient: async () => {
      const db = await getDB()
      await db.clear('queries')
    },
  },
  maxAge: 24 * 60 * 60 * 1000,
})

function ThemedToaster() {
  const { resolvedTheme } = useTheme()
  return <Toaster position="top-right" richColors closeButton theme={resolvedTheme as 'light' | 'dark'} />
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <ErrorBoundary>
          <StoreProvider>
            <WalletProvider>
              <AuthProvider>
                <ContractProvider>
                  <App />
                  <ThemedToaster />
                </ContractProvider>
              </AuthProvider>
            </WalletProvider>
          </StoreProvider>
        </ErrorBoundary>
      </ThemeProvider>
      {import.meta.env.DEV && <ReactQueryDevtools initialIsOpen={false} />}
    </QueryClientProvider>
  </React.StrictMode>
)
