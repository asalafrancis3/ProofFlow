import React, { ReactNode } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import { StoreProvider } from '@/store'

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  })
}

interface WrapperProps {
  children: ReactNode
  initialEntries?: string[]
}

function TestWrapper({ children, initialEntries = ['/'] }: WrapperProps) {
  const queryClient = createTestQueryClient()
  return (
    <QueryClientProvider client={queryClient}>
      <StoreProvider>
        <MemoryRouter initialEntries={initialEntries}>
          {children}
        </MemoryRouter>
      </StoreProvider>
    </QueryClientProvider>
  )
}

function renderWithProviders(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'> & { initialEntries?: string[] }
) {
  const { initialEntries, ...renderOptions } = options ?? {}
  return render(ui, {
    wrapper: ({ children }) => <TestWrapper initialEntries={initialEntries}>{children}</TestWrapper>,
    ...renderOptions,
  })
}

/**
 * Render a page component that uses useParams/useNavigate with proper Route context.
 */
function renderPage(
  component: React.ReactElement,
  path: string,
  initialEntries?: string[]
) {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>
      <StoreProvider>
        <MemoryRouter initialEntries={initialEntries ?? [path]}>
          <Routes>
            <Route path={path} element={component} />
          </Routes>
        </MemoryRouter>
      </StoreProvider>
    </QueryClientProvider>
  )
}

export { renderWithProviders, renderPage, TestWrapper, createTestQueryClient }
