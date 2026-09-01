import { Link } from 'react-router-dom'
import { Activity, Briefcase, DollarSign, Shield, CheckCircle2 } from 'lucide-react'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useJobs } from '@/hooks/useProofFlow'
import { useWallet } from '@/context/WalletContext'

function activityIcon(status: string) {
  switch (status) {
    case 'settled': return <CheckCircle2 className="h-4 w-4 text-green-500" />
    case 'active': return <Briefcase className="h-4 w-4 text-blue-500" />
    case 'funded': return <DollarSign className="h-4 w-4 text-yellow-500" />
    case 'disputed': return <Shield className="h-4 w-4 text-orange-500" />
    default: return <Activity className="h-4 w-4 text-muted-foreground" />
  }
}

export function ActivityPage() {
  useAppTitle('Activity — ProofFlow')
  const { address } = useWallet()
  const { data: jobs, isLoading } = useJobs()

  const myJobs = jobs?.filter((j) => j.client === address || j.status !== 'draft') ?? []

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Activity</h1>
        <p className="text-sm text-muted-foreground">Transaction and job history</p>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="h-16 animate-pulse rounded-lg bg-muted" />
          ))}
        </div>
      ) : myJobs.length === 0 ? (
        <div className="rounded-lg border p-12 text-center">
          <Activity className="mx-auto h-8 w-8 text-muted-foreground" />
          <p className="mt-2 text-sm text-muted-foreground">No activity yet</p>
        </div>
      ) : (
        <div className="space-y-2">
          {myJobs.map((job) => (
            <Link
              key={job.id}
              to={`/jobs/${job.id}`}
              className="flex items-center gap-3 rounded-lg border p-3 transition-colors hover:bg-accent"
            >
              {activityIcon(job.status)}
              <div className="flex-1">
                <p className="text-sm font-medium">{job.title}</p>
                <p className="text-xs text-muted-foreground">
                  #{job.id} · {job.status} · {job.milestone_count} milestones
                </p>
              </div>
              <span className="text-xs text-muted-foreground">
                {new Date(job.created_at * 1000).toLocaleDateString()}
              </span>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
