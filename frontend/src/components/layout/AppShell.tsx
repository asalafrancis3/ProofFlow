import type { PropsWithChildren } from 'react'
import { NavLink } from 'react-router-dom'
import {
  Home,
  Briefcase,
  PlusCircle,
  Shield,
  User,
  LogOut,
  Activity,
  Wallet,
  Settings,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { useWallet } from '@/context/WalletContext'
import { useAuth } from '@/context/AuthContext'
import { Button } from '@/components/ui/Button'
import { ThemeToggle } from '@/components/ui/ThemeToggle'
import { OfflineIndicator } from '@/components/OfflineIndicator'

const NAV_LINKS = [
  { label: 'Dashboard', href: '/dashboard', roles: ['client', 'worker', 'verifier', 'admin'], icon: Home },
  { label: 'Jobs', href: '/jobs', roles: ['client', 'worker', 'verifier'], icon: Briefcase },
  { label: 'Create Job', href: '/jobs/new', roles: ['client'], icon: PlusCircle },
  { label: 'Verification', href: '/verification', roles: ['verifier'], icon: Shield },
  { label: 'Activity', href: '/activity', roles: ['client', 'worker', 'verifier'], icon: Activity },
  { label: 'Settings', href: '/settings', roles: ['client', 'worker', 'verifier', 'admin'], icon: Settings },
  { label: 'Admin', href: '/admin', roles: ['admin'], icon: Shield },
]

function truncate(addr: string) {
  return `${addr.slice(0, 4)}...${addr.slice(-4)}`
}

export function AppShell({ children }: PropsWithChildren) {
  const { address, isConnected, connect, disconnect, isLoading } = useWallet()
  const { user, logout } = useAuth()

  const role = user?.role ?? ''
  const links = NAV_LINKS.filter((l) => !role || l.roles.includes(role))

  const Sidebar = (
    <nav className="flex flex-col gap-1 p-4" aria-label="Main navigation">
      <div className="mb-4 flex items-center gap-2 px-2">
        <Shield className="h-6 w-6 text-primary" aria-hidden="true" />
        <span className="text-lg font-bold">ProofFlow</span>
      </div>
      {links.map((link) => {
        const Icon = link.icon
        return (
          <NavLink
            key={link.href}
            to={link.href}
            className={({ isActive }) =>
              cn(
                'flex min-h-11 items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground',
                isActive ? 'bg-accent text-accent-foreground' : 'text-foreground'
              )
            }
          >
            <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
            {link.label}
          </NavLink>
        )
      })}
      {user && (
        <button
          onClick={() => logout()}
          aria-label="Sign out"
          className="mt-auto flex min-h-11 items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-destructive transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <LogOut className="h-4 w-4" aria-hidden="true" />
          Sign out
        </button>
      )}
    </nav>
  )

  return (
    <div className="flex min-h-screen bg-background text-foreground">
      {/* Skip to main content link for keyboard users */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-[100] focus:rounded-md focus:bg-background focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:ring-2 focus:ring-ring"
      >
        Skip to main content
      </a>

      {/* Desktop sidebar */}
      <aside className="hidden w-56 shrink-0 border-r md:flex md:flex-col">{Sidebar}</aside>

      <div className="flex flex-1 flex-col">
        {/* Header */}
        <header className="flex h-14 items-center justify-between border-b px-4">
          <span className="text-sm font-medium md:hidden">ProofFlow</span>

          <div className="ml-auto flex items-center gap-3">
            <ThemeToggle className="shrink-0" />

            {isConnected && address ? (
              <div className="flex items-center gap-2">
                <span className="hidden items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-medium sm:flex">
                  <Wallet className="h-3.5 w-3.5 text-primary" aria-hidden="true" />
                  <span aria-label={`Connected wallet: ${address}`}>{truncate(address)}</span>
                </span>
                <Button variant="ghost" size="sm" onClick={disconnect} aria-label="Disconnect wallet">
                  Disconnect
                </Button>
              </div>
            ) : (
              <Button size="sm" onClick={connect} disabled={isLoading} aria-label="Connect wallet">
                {isLoading ? 'Connecting…' : 'Connect Wallet'}
              </Button>
            )}
          </div>
        </header>

        <OfflineIndicator />
        <main id="main-content" className="flex-1 overflow-x-hidden p-4 pb-20 sm:p-6 sm:pb-6">
          {children}
        </main>
      </div>

      {/* Mobile bottom navigation */}
      <nav
        className="fixed inset-x-0 bottom-0 z-40 border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 md:hidden"
        aria-label="Mobile navigation"
      >
        <div className="flex min-h-16 items-center justify-around gap-1 px-2 py-1">
          {links.slice(0, 4).map((link) => {
            const Icon = link.icon
            return (
              <NavLink
                key={link.href}
                to={link.href}
                className={({ isActive }) =>
                  cn(
                    'flex min-h-12 min-w-[3.5rem] flex-1 flex-col items-center justify-center rounded-md px-2 py-1 text-[10px] font-medium transition-colors',
                    isActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'
                  )
                }
              >
                <Icon className="mb-0.5 h-5 w-5" aria-hidden="true" />
                <span className="truncate">{link.label}</span>
              </NavLink>
            )
          })}
          <NavLink
            to="/settings"
            className={({ isActive }) =>
              cn(
                'flex min-h-12 min-w-[3.5rem] flex-1 flex-col items-center justify-center rounded-md px-2 py-1 text-[10px] font-medium transition-colors',
                isActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'
              )
            }
          >
            <User className="mb-0.5 h-5 w-5" aria-hidden="true" />
            <span className="truncate">Settings</span>
          </NavLink>
        </div>
      </nav>
    </div>
  )
}
