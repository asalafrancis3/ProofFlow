import { useParams, Link } from 'react-router-dom'
import { ArrowLeft, Shield, CheckCircle2, AlertTriangle, Award } from 'lucide-react'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useReputation } from '@/hooks/useProofFlow'

export function ReputationPage() {
  const { address } = useParams<{ address: string }>()
  useAppTitle(`Reputation — ProofFlow`)
  const { data: rep, isLoading } = useReputation(address ?? '')

  if (isLoading) {
    return <div className="space-y-4">{[1, 2, 3].map((i) => <div key={i} className="h-20 animate-pulse rounded-lg bg-muted" />)}</div>
  }

  if (!rep) {
    return (
      <div className="text-center">
        <Shield className="mx-auto h-8 w-8 text-muted-foreground" />
        <p className="mt-2 text-muted-foreground">No reputation data found</p>
        <Link to="/dashboard" className="text-sm text-primary hover:underline">Back to Dashboard</Link>
      </div>
    )
  }

  const scoreColor =
    rep.score >= 80 ? 'text-green-600' :
    rep.score >= 50 ? 'text-yellow-600' :
    'text-red-600'

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <Link to="/dashboard" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="h-4 w-4" /> Back
      </Link>

      {/* Profile header */}
      <div className="rounded-lg border p-6 text-center">
        <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
          <Shield className="h-8 w-8 text-primary" />
        </div>
        <h1 className="mt-4 text-xl font-bold">{address?.slice(0, 12)}…</h1>
        <p className="text-sm text-muted-foreground">{address}</p>
        <div className="mt-4">
          <p className={`text-4xl font-bold ${scoreColor}`}>{rep.score}</p>
          <p className="text-sm text-muted-foreground">Reputation Score</p>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <div className="rounded-lg border p-4 text-center">
          <CheckCircle2 className="mx-auto h-5 w-5 text-green-500" />
          <p className="mt-2 text-2xl font-bold">{rep.completed_jobs}</p>
          <p className="text-xs text-muted-foreground">Completed Jobs</p>
        </div>
        <div className="rounded-lg border p-4 text-center">
          <Award className="mx-auto h-5 w-5 text-blue-500" />
          <p className="mt-2 text-2xl font-bold">{rep.successful_attestations}</p>
          <p className="text-xs text-muted-foreground">Attestations</p>
        </div>
        <div className="rounded-lg border p-4 text-center">
          <AlertTriangle className="mx-auto h-5 w-5 text-orange-500" />
          <p className="mt-2 text-2xl font-bold">{rep.disputes_involved}</p>
          <p className="text-xs text-muted-foreground">Disputes</p>
        </div>
        <div className="rounded-lg border p-4 text-center">
          <Shield className="mx-auto h-5 w-5 text-primary" />
          <p className="mt-2 text-2xl font-bold">{rep.disputes_won}</p>
          <p className="text-xs text-muted-foreground">Disputes Won</p>
        </div>
      </div>

      {/* Earnings */}
      <div className="rounded-lg border p-4">
        <h2 className="text-sm font-semibold uppercase text-muted-foreground">Earnings</h2>
        <p className="mt-2 text-3xl font-bold">{rep.total_earned.toLocaleString()}</p>
        <p className="text-xs text-muted-foreground">Total tokens earned</p>
      </div>
    </div>
  )
}
