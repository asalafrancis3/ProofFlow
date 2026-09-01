import { ShieldAlert, AlertTriangle } from 'lucide-react'
import { useAuth } from '@/context/AuthContext'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useAdminTabs } from '@/hooks/useAdminTabs'
import {
  OverviewTab,
  UsersTab,
  DisputesTab,
  SystemHealthTab,
  ConfigTab,
  AuditLogTab,
} from '@/components/admin'

export function AdminDashboardPage() {
  useAppTitle('Admin Dashboard')
  const { user } = useAuth()
  const isAdmin = user?.role === 'Admin'
  const { activeTab, setActiveTab, visibleTabs } = useAdminTabs({ isAdmin })

  return (
    <div className="space-y-6 px-4 py-6 sm:space-y-8 sm:py-8">
      <div className="flex items-center gap-3">
        <ShieldAlert className="h-6 w-6 text-destructive" />
        <div>
          <h1 className="text-xl font-bold sm:text-2xl">Admin Dashboard</h1>
          <p className="mt-0.5 text-sm text-muted-foreground">System management and oversight.</p>
        </div>
      </div>

      {!isAdmin && (
        <div className="flex items-center gap-2 rounded-md border border-yellow-400 bg-yellow-50 px-4 py-2 text-sm text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          Some sections are restricted to admin accounts only.
        </div>
      )}

      {/* Tab bar */}
      <div
        role="tablist"
        aria-label="Admin sections"
        className="flex flex-wrap gap-1 rounded-lg border bg-muted p-1"
      >
        {visibleTabs.map((tab) => (
          <button
            key={tab.id}
            role="tab"
            aria-selected={activeTab === tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
              activeTab === tab.id
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab panels */}
      <div role="tabpanel">
        {activeTab === 'overview' && <OverviewTab />}
        {activeTab === 'users' && <UsersTab />}
        {activeTab === 'disputes' && <DisputesTab />}
        {activeTab === 'health' && <SystemHealthTab />}
        {activeTab === 'config' && isAdmin && <ConfigTab />}
        {activeTab === 'audit' && <AuditLogTab />}
      </div>
    </div>
  )
}

