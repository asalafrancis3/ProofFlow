import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Shield, Wallet } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/Select'
import { useWallet } from '@/context/WalletContext'
import { useAuth } from '@/context/AuthContext'

const ROLES = [
  { value: 'Client', label: 'Client — Post bounties & milestones' },
  { value: 'Worker', label: 'Worker — Submit proofs & evidence' },
  { value: 'Verifier', label: 'Verifier — Attest to claims' },
]

export function LoginPage() {
  useAppTitle('ProofFlow — Sign In')
  const navigate = useNavigate()
  const { address, isConnected, connect, isLoading: walletLoading, error: walletError } = useWallet()
  const { login } = useAuth()

  const [role, setRole] = useState('Worker')

  function handleConnect() {
    if (!isConnected) {
      connect()
      return
    }
    if (!address) return
    login({ address, role, name: role })
    navigate('/dashboard', { replace: true })
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4">
      <div className="w-full max-w-sm space-y-6">
        <div className="flex flex-col items-center gap-2 text-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <Shield className="h-6 w-6 text-primary" />
          </div>
          <h1 className="text-2xl font-bold">Welcome to ProofFlow</h1>
          <p className="text-sm text-muted-foreground">
            Connect your Freighter wallet to get started.
          </p>
        </div>

        {!isConnected && (
          <div className="space-y-3">
            <Button className="w-full" size="lg" onClick={handleConnect} disabled={walletLoading}>
              <Wallet className="mr-2 h-4 w-4" />
              {walletLoading ? 'Connecting...' : 'Connect Wallet'}
            </Button>
            {walletError && (
              <p role="alert" aria-live="assertive" className="text-center text-sm text-destructive">
                {walletError}
              </p>
            )}
          </div>
        )}

        {isConnected && (
          <div className="space-y-4">
            <p className="text-center text-sm text-muted-foreground">
              Select your role to continue.
            </p>

            <div className="space-y-1">
              <label htmlFor="role-select" className="text-sm font-medium">
                Role
              </label>
              <Select value={role} onValueChange={(v) => setRole(v)}>
                <SelectTrigger id="role-select" className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {ROLES.map((r) => (
                    <SelectItem key={r.value} value={r.value}>
                      {r.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Button className="w-full" onClick={handleConnect}>
              Enter ProofFlow
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}

function useAppTitle(title: string) {
  if (typeof document !== 'undefined') {
    document.title = title
  }
}
