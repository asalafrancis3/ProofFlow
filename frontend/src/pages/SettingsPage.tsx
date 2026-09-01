import { useState } from 'react'
import { Copy, Check, LogOut, Sun, Moon, Monitor } from 'lucide-react'
import { useWallet } from '@/context/WalletContext'
import { useContract } from '@/context/ContractContext'
import { useAuth } from '@/context/AuthContext'
import { useTheme } from '@/context/ThemeProvider'
import { NotificationStore, type NotificationPreferences, type NotificationType } from '@/lib/notifications'
import { NETWORK_CONFIGS } from '@/lib/stellar'
import { Button } from '@/components/ui/Button'
import { Switch } from '@/components/ui/Switch'
import { LanguageSelector } from '@/components/ui/LanguageSelector'
import { cn } from '@/lib/utils'
import { useOnboarding } from '@/hooks/useOnboarding'

// ── Section wrapper ───────────────────────────────────────────────────────────

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
      </h2>
      <div className="rounded-lg border divide-y divide-border">{children}</div>
    </div>
  )
}

function Row({
  label,
  description,
  children
}: {
  label: string
  description?: string
  children: React.ReactNode
}) {
  return (
    <div className="flex flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="text-sm font-medium">{label}</p>
        {description && <p className="text-xs text-muted-foreground">{description}</p>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

// ── Network options ───────────────────────────────────────────────────────────

const NETWORKS = ['TESTNET', 'MAINNET', 'FUTURENET', 'STANDALONE'] as const
type Network = (typeof NETWORKS)[number]

// ── Page ─────────────────────────────────────────────────────────────────────

export function SettingsPage() {
  const { address, disconnect } = useWallet()
  const { logout } = useAuth()
  const { config, updateConfig } = useContract()
  const { theme, isDark, isReady, setTheme } = useTheme()

  const [copied, setCopied] = useState(false)
  const [notifPrefs, setNotifPrefs] = useState<NotificationPreferences>(() =>
    NotificationStore.loadPrefs()
  )

  function copyAddress() {
    if (!address) return
    navigator.clipboard.writeText(address).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  function handleDisconnect() {
    disconnect()
    logout()
  }

  function handleNetworkChange(network: Network) {
    const netCfg = NETWORK_CONFIGS[network]
    updateConfig({ network, rpcUrl: netCfg.rpcUrl })
  }

  const { resetOnboarding } = useOnboarding()

  function handleNotifToggle(type: NotificationType, checked: boolean) {
    const updated = { ...notifPrefs, [type]: checked }
    setNotifPrefs(updated)
    NotificationStore.savePrefs(updated)
  }

  return (
    <div className="mx-auto max-w-xl space-y-6 px-4 py-6 sm:space-y-8 sm:py-8">
      <div>
        <h1 className="text-xl font-bold sm:text-2xl">Settings</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Manage your wallet, appearance, and network.
        </p>
      </div>

      {/* Wallet */}
      <Section title="Wallet">
        <Row label="Connected address" description={address ? undefined : 'No wallet connected'}>
          {address ? (
            <div className="flex items-center gap-2">
              <span className="max-w-[160px] truncate font-mono text-xs">{address}</span>
              <button
                type="button"
                onClick={copyAddress}
                aria-label="Copy address"
                className="rounded-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              >
                {copied ? (
                  <Check className="h-4 w-4 text-green-500" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </button>
              <span role="status" aria-live="polite" className="sr-only">
                {copied ? 'Address copied to clipboard.' : ''}
              </span>
            </div>
          ) : (
            <span className="text-xs text-muted-foreground">—</span>
          )}
        </Row>

        <Row label="Disconnect" description="Sign out and disconnect your wallet">
          <Button variant="destructive" size="sm" onClick={handleDisconnect} disabled={!address}>
            <LogOut className="mr-1.5 h-3.5 w-3.5" />
            Disconnect
          </Button>
        </Row>
      </Section>

      {/* Appearance */}
      <Section title="Appearance">
        <Row label="Dark mode" description="Switch between light and dark theme">
          <div className="flex items-center gap-2">
            <Sun className="h-4 w-4 text-muted-foreground" />
            <Switch
              checked={isReady && isDark && theme !== 'high-contrast'}
              onCheckedChange={(checked) => setTheme(checked ? 'dark' : 'light')}
              aria-label="Toggle dark mode"
              disabled={theme === 'high-contrast'}
            />
            <Moon className="h-4 w-4 text-muted-foreground" />
          </div>
        </Row>

        <Row label="High contrast mode" description="For improved accessibility">
          <Switch
            checked={theme === 'high-contrast'}
            onCheckedChange={(checked) => setTheme(checked ? 'high-contrast' : isDark ? 'dark' : 'light')}
            aria-label="Toggle high contrast mode"
          />
        </Row>

        <Row label="System theme" description="Follow your OS preference">
          <Button
            variant={theme === 'system' ? 'secondary' : 'outline'}
            size="sm"
            onClick={() => setTheme('system')}
            className="w-full sm:w-auto"
            disabled={theme === 'high-contrast'}
          >
            <Monitor className="mr-1.5 h-3.5 w-3.5" />
            Use system
          </Button>
        </Row>
      </Section>

      {/* Language */}
      <Section title="Language">
        <Row label="Display language" description="Choose your preferred language (Arabic enables RTL)">
          <LanguageSelector />
        </Row>
      </Section>

      {/* Network */}
      <Section title="Network">
        <div className="px-4 py-3 space-y-2">
          <p className="text-sm font-medium">Stellar network</p>
          <p className="text-xs text-muted-foreground">
            Changing the network updates the RPC endpoint. Requires a contract deployed on the
            selected network.
          </p>
          <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4">
            {NETWORKS.map((net) => (
              <button
                key={net}
                type="button"
                onClick={() => handleNetworkChange(net)}
                className={cn(
                  'rounded-md border px-3 py-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                  config.network === net
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border bg-background text-muted-foreground hover:border-primary/50 hover:text-foreground'
                )}
              >
                {net.charAt(0) + net.slice(1).toLowerCase()}
              </button>
            ))}
          </div>
          <p className="pt-1 text-xs text-muted-foreground">
            RPC: <span className="font-mono">{config.rpcUrl}</span>
          </p>
        </div>
      </Section>

      {/* Notifications */}
      <Section title="Notifications">
        <Row
          label="Transfer notifications"
          description="Notify when milestones are submitted or approved"
        >
          <Switch
            checked={notifPrefs.transfer}
            onCheckedChange={(checked) => handleNotifToggle('transfer', checked)}
            aria-label="Toggle transfer notifications"
          />
        </Row>
        <Row
          label="Confirmation notifications"
          description="Notify when payments are released"
        >
          <Switch
            checked={notifPrefs.confirmation}
            onCheckedChange={(checked) => handleNotifToggle('confirmation', checked)}
            aria-label="Toggle confirmation notifications"
          />
        </Row>
        <Row
          label="Reward notifications"
          description="Notify when you earn tokens"
        >
          <Switch
            checked={notifPrefs.reward}
            onCheckedChange={(checked) => handleNotifToggle('reward', checked)}
            aria-label="Toggle reward notifications"
          />
        </Row>
        <Row
          label="System notifications"
          description="Notify about platform announcements"
        >
          <Switch
            checked={notifPrefs.system}
            onCheckedChange={(checked) => handleNotifToggle('system', checked)}
            aria-label="Toggle system notifications"
          />
        </Row>
      </Section>

      {/* Onboarding */}
      <Section title="Onboarding">
        <Row
          label="Restart tutorial"
          description="Start the interactive onboarding tutorial again"
        >
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              resetOnboarding()
              // Show a toast or notification that tutorial will restart on next login
              alert('Tutorial will restart on your next login!')
            }}
          >
            Restart Tutorial
          </Button>
        </Row>
      </Section>
    </div>
  )
}
