import { lazy, Suspense } from 'react'
import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom'
import { useAuth } from '@/context/AuthContext'
import { AppShell } from '@/components/layout/AppShell'
import { PageSkeleton } from '@/components/ui/Skeletons'
import { RouteErrorBoundary } from '@/components/RouteErrorBoundary'

// Eagerly loaded pages
import { LandingPage } from '@/pages/LandingPage'
import { LoginPage } from '@/pages/LoginPage'
import { NotFoundPage } from '@/pages/NotFoundPage'

// Lazy-loaded ProofFlow pages
const DashboardPage = lazy(() =>
  import('@/pages/DashboardPage').then((m) => ({ default: m.DashboardPage }))
)
const JobsPage = lazy(() =>
  import('@/pages/JobsPage').then((m) => ({ default: m.JobsPage }))
)
const CreateJobPage = lazy(() =>
  import('@/pages/CreateJobPage').then((m) => ({ default: m.CreateJobPage }))
)
const JobDetailPage = lazy(() =>
  import('@/pages/JobDetailPage').then((m) => ({ default: m.JobDetailPage }))
)
const VerificationPage = lazy(() =>
  import('@/pages/VerificationPage').then((m) => ({ default: m.VerificationPage }))
)
const ReputationPage = lazy(() =>
  import('@/pages/ReputationPage').then((m) => ({ default: m.ReputationPage }))
)
const ActivityPage = lazy(() =>
  import('@/pages/ActivityPage').then((m) => ({ default: m.ActivityPage }))
)
const SettingsPage = lazy(() =>
  import('@/pages/SettingsPage').then((m) => ({ default: m.SettingsPage }))
)
const AdminDashboardPage = lazy(() =>
  import('@/pages/AdminDashboardPage').then((m) => ({ default: m.AdminDashboardPage }))
)

// Infrastructure pages (keep as-is)
const OfflinePage = lazy(() =>
  import('@/pages/OfflinePage').then((m) => ({ default: m.OfflinePage }))
)
const OfflineSettings = lazy(() =>
  import('@/pages/OfflineSettings').then((m) => ({ default: m.OfflineSettings }))
)
const FeatureFlagsPage = lazy(() =>
  import('@/pages/FeatureFlagsPage').then((m) => ({ default: m.FeatureFlagsPage }))
)
const TestReportsPage = lazy(() =>
  import('@/pages/TestReportsPage').then((m) => ({ default: m.TestReportsPage }))
)
const QRCodePage = lazy(() =>
  import('@/pages/QRCodePage').then((m) => ({ default: m.QRCodePage }))
)

// Suspense wrapper
// eslint-disable-next-line react-refresh/only-export-components
function PageFallback() {
  return (
    <Suspense fallback={<PageSkeleton />}>
      <Outlet />
    </Suspense>
  )
}

// Auth guard
// eslint-disable-next-line react-refresh/only-export-components
function ProtectedLayout() {
  const { isAuthenticated, isLoading } = useAuth()
  if (isLoading) return null
  return isAuthenticated ? (
    <AppShell>
      <PageFallback />
    </AppShell>
  ) : (
    <Navigate to="/login" replace />
  )
}

// ProofFlow protected routes
const PROTECTED_ROUTES = [
  // Core ProofFlow
  { path: 'dashboard', element: <DashboardPage /> },
  { path: 'jobs', element: <JobsPage /> },
  { path: 'jobs/new', element: <CreateJobPage /> },
  { path: 'jobs/:id', element: <JobDetailPage /> },
  { path: 'verification', element: <VerificationPage /> },
  { path: 'reputation/:address', element: <ReputationPage /> },
  { path: 'activity', element: <ActivityPage /> },
  // Infrastructure
  { path: 'settings', element: <SettingsPage /> },
  { path: 'settings/offline', element: <OfflineSettings /> },
  { path: 'admin', element: <AdminDashboardPage /> },
  { path: 'offline', element: <OfflinePage /> },
  { path: 'feature-flags', element: <FeatureFlagsPage /> },
  { path: 'test-reports', element: <TestReportsPage /> },
  { path: 'qr', element: <QRCodePage /> },
]

export const router = createBrowserRouter([
  { path: '/', element: <LandingPage />, errorElement: <RouteErrorBoundary /> },
  { path: '/login', element: <LoginPage />, errorElement: <RouteErrorBoundary /> },
  {
    element: <ProtectedLayout />,
    errorElement: <RouteErrorBoundary />,
    children: PROTECTED_ROUTES.map((route) => ({
      ...route,
      errorElement: <RouteErrorBoundary />,
    })),
  },
  { path: '*', element: <NotFoundPage />, errorElement: <RouteErrorBoundary /> },
])
