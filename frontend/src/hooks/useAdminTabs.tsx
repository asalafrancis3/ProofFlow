import { useState, type ReactNode } from 'react'
import {
  Users,
  Settings,
  ShieldAlert,
  Activity,
  AlertTriangle,
  Heart,
} from 'lucide-react'

export type AdminTab =
  | 'overview'
  | 'users'
  | 'disputes'
  | 'health'
  | 'config'
  | 'audit'

export interface AdminTabConfig {
  id: AdminTab
  label: string
  icon: ReactNode
  adminOnly?: boolean
}

export const ADMIN_TABS: AdminTabConfig[] = [
  { id: 'overview', label: 'Overview', icon: <Activity className="h-4 w-4" /> },
  { id: 'users', label: 'Users', icon: <Users className="h-4 w-4" /> },
  { id: 'disputes', label: 'Disputes', icon: <AlertTriangle className="h-4 w-4" /> },
  { id: 'health', label: 'System Health', icon: <Heart className="h-4 w-4" /> },
  { id: 'config', label: 'Config', icon: <Settings className="h-4 w-4" />, adminOnly: true },
  { id: 'audit', label: 'Audit Log', icon: <ShieldAlert className="h-4 w-4" /> },
]

export interface UseAdminTabsOptions {
  initialTab?: AdminTab
  isAdmin?: boolean
  tabs?: AdminTabConfig[]
}

export function useAdminTabs(options: UseAdminTabsOptions = {}) {
  const { initialTab = 'overview', isAdmin = false, tabs = ADMIN_TABS } = options
  const [activeTab, setActiveTab] = useState<AdminTab>(initialTab)

  const visibleTabs = tabs.filter((t) => !t.adminOnly || isAdmin)

  return {
    activeTab,
    setActiveTab,
    visibleTabs,
    allTabs: tabs,
  }
}
