import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '@/test/msw/server'
import { renderWithProviders, renderPage } from '@/test/proofflow-test-utils'

vi.mock('@/hooks/useAppTitle', () => ({
  useAppTitle: vi.fn(),
}))

const MOCK_ADDRESS = 'GAAA111111111111111111111111111111111111111111111111'

vi.mock('@/context/WalletContext', () => ({
  useWallet: () => ({
    address: MOCK_ADDRESS,
    isConnected: true,
    connect: vi.fn(),
    isLoading: false,
    error: null,
  }),
}))

beforeEach(() => {
  if (typeof localStorage !== 'undefined') localStorage.clear()
})

// ══════════════════════════════════════════════════════════════════════════════
// 1. Dashboard — loads stats
// ══════════════════════════════════════════════════════════════════════════════

describe('DashboardPage', () => {
  it('shows loading state then renders stats', async () => {
    const { DashboardPage } = await import('@/pages/DashboardPage')
    renderWithProviders(<DashboardPage />)
    expect(screen.getByText('Dashboard')).toBeInTheDocument()
    await waitFor(() => {
      expect(screen.getByText('My Jobs')).toBeInTheDocument()
    })
  })

  it('shows empty state when no jobs', async () => {
    server.use(
      http.get('*/api/v1/jobs', () => {
        return HttpResponse.json({ success: true, data: [], error: null })
      })
    )
    const { DashboardPage } = await import('@/pages/DashboardPage')
    renderWithProviders(<DashboardPage />)
    await waitFor(() => {
      expect(screen.getByText('No jobs yet')).toBeInTheDocument()
    })
    server.resetHandlers()
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 2. Jobs listing
// ══════════════════════════════════════════════════════════════════════════════

describe('JobsPage', () => {
  it('renders job list', async () => {
    const { JobsPage } = await import('@/pages/JobsPage')
    renderWithProviders(<JobsPage />)
    await waitFor(() => {
      expect(screen.getByText('Smart Contract Audit')).toBeInTheDocument()
    })
    expect(screen.getByText('Frontend Development')).toBeInTheDocument()
  })

  it('shows empty state when no jobs', async () => {
    server.use(
      http.get('*/api/v1/jobs', () => {
        return HttpResponse.json({ success: true, data: [], error: null })
      })
    )
    const { JobsPage } = await import('@/pages/JobsPage')
    renderWithProviders(<JobsPage />)
    await waitFor(() => {
      expect(screen.getByText('No jobs found')).toBeInTheDocument()
    })
    server.resetHandlers()
  })

  it('filters jobs by status', async () => {
    const user = userEvent.setup()
    const { JobsPage } = await import('@/pages/JobsPage')
    renderWithProviders(<JobsPage />)
    await waitFor(() => {
      expect(screen.getByText('Smart Contract Audit')).toBeInTheDocument()
    })
    await user.click(screen.getByText('Funded'))
    await waitFor(() => {
      expect(screen.getByText('Frontend Development')).toBeInTheDocument()
      expect(screen.queryByText('Smart Contract Audit')).not.toBeInTheDocument()
    })
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 3. Create Job
// ══════════════════════════════════════════════════════════════════════════════

describe('CreateJobPage', () => {
  it('renders the create form', async () => {
    const { CreateJobPage } = await import('@/pages/CreateJobPage')
    renderWithProviders(<CreateJobPage />)
    await waitFor(() => {
      expect(screen.getByText('Create Job', { selector: 'h1' })).toBeInTheDocument()
    })
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 4. Job Detail
// ══════════════════════════════════════════════════════════════════════════════

describe('JobDetailPage', () => {
  it('renders job details', async () => {
    const { JobDetailPage } = await import('@/pages/JobDetailPage')
    renderPage(<JobDetailPage />, '/jobs/:id', ['/jobs/1'])
    await waitFor(() => {
      expect(screen.getByText('Smart Contract Audit')).toBeInTheDocument()
    })
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 5. Verification
// ══════════════════════════════════════════════════════════════════════════════

describe('VerificationPage', () => {
  it('renders the verification queue', async () => {
    const { VerificationPage } = await import('@/pages/VerificationPage')
    renderWithProviders(<VerificationPage />)
    await waitFor(() => {
      expect(screen.getByText(/Verification/i)).toBeInTheDocument()
    })
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 6. Reputation
// ══════════════════════════════════════════════════════════════════════════════

describe('ReputationPage', () => {
  it('renders reputation data', async () => {
    const { ReputationPage } = await import('@/pages/ReputationPage')
    renderPage(
      <ReputationPage />,
      '/reputation/:address',
      ['/reputation/GAAA111111111111111111111111111111111111111111111111']
    )
    await waitFor(() => {
      expect(screen.getByText('850')).toBeInTheDocument()
    })
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 7. API failure
// ══════════════════════════════════════════════════════════════════════════════

describe('API failure handling', () => {
  it('handles API error gracefully', async () => {
    server.use(
      http.get('*/api/v1/jobs', () => {
        return HttpResponse.json({ success: false, data: null, error: 'Server error' }, { status: 500 })
      })
    )
    const { JobsPage } = await import('@/pages/JobsPage')
    renderWithProviders(<JobsPage />)
    await waitFor(() => {
      expect(screen.getByText('No jobs found')).toBeInTheDocument()
    })
    server.resetHandlers()
  })
})

// ══════════════════════════════════════════════════════════════════════════════
// 8. Login page
// ══════════════════════════════════════════════════════════════════════════════

describe('LoginPage', () => {
  it('shows ProofFlow branding', async () => {
    const { LoginPage } = await import('@/pages/LoginPage')
    renderWithProviders(<LoginPage />)
    await waitFor(() => {
      expect(screen.getByText(/Welcome to ProofFlow/i)).toBeInTheDocument()
    })
  })
})
