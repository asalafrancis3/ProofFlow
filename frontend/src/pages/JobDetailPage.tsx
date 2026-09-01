import { useParams, Link } from 'react-router-dom'
import { ArrowLeft, DollarSign, CheckCircle2, Clock, AlertTriangle, FileCheck } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useJob, useEscrow, useFundJob } from '@/hooks/useProofFlow'
import { useWallet } from '@/context/WalletContext'

function statusIcon(status: string) {
  switch (status) {
    case 'active': return <Clock className="h-4 w-4 text-green-500" />
    case 'funded': return <DollarSign className="h-4 w-4 text-blue-500" />
    case 'settled': return <CheckCircle2 className="h-4 w-4 text-gray-500" />
    case 'disputed': return <AlertTriangle className="h-4 w-4 text-orange-500" />
    default: return <Clock className="h-4 w-4 text-muted-foreground" />
  }
}

function milestoneStatusColor(status: string): string {
  switch (status) {
    case 'approved': return 'text-green-600'
    case 'submitted': return 'text-yellow-600'
    case 'rejected': return 'text-red-600'
    case 'released': return 'text-blue-600'
    default: return 'text-muted-foreground'
  }
}

export function JobDetailPage() {
  const { id } = useParams<{ id: string }>()
  const jobId = Number(id)
  useAppTitle(`Job #${id} — ProofFlow`)

  const { data: job, isLoading } = useJob(jobId)
  const { data: escrow } = useEscrow(jobId)
  const { address } = useWallet()
  const fundJob = useFundJob()

  if (isLoading) {
    return <div className="space-y-4">{[1, 2, 3].map((i) => <div key={i} className="h-20 animate-pulse rounded-lg bg-muted" />)}</div>
  }

  if (!job) {
    return (
      <div className="text-center">
        <p className="text-muted-foreground">Job not found</p>
        <Link to="/jobs"><Button variant="link">Back to Jobs</Button></Link>
      </div>
    )
  }

  const isClient = address && job.client === address
  const canFund = isClient && job.status === 'draft'

  return (
    <div className="space-y-6">
      <Link to="/jobs" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="h-4 w-4" /> Back to Jobs
      </Link>

      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold">{job.title}</h1>
          <p className="text-sm text-muted-foreground">Job #{job.id} · {job.client.slice(0, 8)}…</p>
        </div>
        <div className="flex items-center gap-2">
          {statusIcon(job.status)}
          <span className="text-sm font-medium capitalize">{job.status}</span>
        </div>
      </div>

      {/* Description */}
      <div className="rounded-lg border p-4">
        <h2 className="text-sm font-semibold uppercase text-muted-foreground">Description</h2>
        <p className="mt-2 text-sm">{job.description || 'No description provided.'}</p>
      </div>

      {/* Escrow */}
      {escrow && (
        <div className="rounded-lg border p-4">
          <h2 className="text-sm font-semibold uppercase text-muted-foreground">Escrow</h2>
          <div className="mt-3 grid grid-cols-3 gap-4 text-center">
            <div>
              <p className="text-2xl font-bold">{escrow.total_funded.toLocaleString()}</p>
              <p className="text-xs text-muted-foreground">Funded</p>
            </div>
            <div>
              <p className="text-2xl font-bold text-green-600">{escrow.total_released.toLocaleString()}</p>
              <p className="text-xs text-muted-foreground">Released</p>
            </div>
            <div>
              <p className="text-2xl font-bold text-yellow-600">{escrow.total_frozen.toLocaleString()}</p>
              <p className="text-xs text-muted-foreground">Frozen</p>
            </div>
          </div>
          {canFund && (
            <Button
              className="mt-4 w-full"
              onClick={() => fundJob.mutate(jobId)}
              disabled={fundJob.isPending}
            >
              {fundJob.isPending ? 'Funding…' : 'Fund Escrow'}
            </Button>
          )}
        </div>
      )}

      {/* Milestones */}
      <div>
        <h2 className="text-lg font-semibold">Milestones</h2>
        <p className="text-sm text-muted-foreground">{job.milestone_count} milestones defined</p>
        <div className="mt-4 space-y-3">
          {Array.from({ length: job.milestone_count }, (_, i) => (
            <div key={i} className="flex items-center justify-between rounded-lg border p-4">
              <div className="flex items-center gap-3">
                <FileCheck className="h-5 w-5 text-muted-foreground" />
                <div>
                  <p className="font-medium">Milestone {i + 1}</p>
                  <p className={`text-xs ${milestoneStatusColor('pending')}`}>
                    Pending
                  </p>
                </div>
              </div>
              <Button variant="ghost" size="sm" asChild>
                <Link to={`/jobs/${jobId}/milestones/${i}`}>View</Link>
              </Button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
