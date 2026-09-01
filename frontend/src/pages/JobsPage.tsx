import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Briefcase, Plus } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useJobs } from '@/hooks/useProofFlow'
import type { JobStatus } from '@/api/proofflow'

const STATUS_FILTERS: { label: string; value: string }[] = [
  { label: 'All', value: 'all' },
  { label: 'Funded', value: 'funded' },
  { label: 'Active', value: 'active' },
  { label: 'In Review', value: 'in_review' },
  { label: 'Settled', value: 'settled' },
  { label: 'Disputed', value: 'disputed' },
]

function statusColor(status: JobStatus): string {
  switch (status) {
    case 'funded': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300'
    case 'active': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300'
    case 'in_review': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300'
    case 'settled': return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300'
    case 'cancelled': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
    case 'disputed': return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-300'
    default: return 'bg-gray-100 text-gray-800'
  }
}

export function JobsPage() {
  useAppTitle('Jobs — ProofFlow')
  const [statusFilter, setStatusFilter] = useState('all')
  const { data: jobs, isLoading } = useJobs(statusFilter === 'all' ? undefined : statusFilter)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Jobs</h1>
          <p className="text-sm text-muted-foreground">Browse and manage work agreements</p>
        </div>
        <Link to="/jobs/new">
          <Button>
            <Plus className="mr-2 h-4 w-4" />
            Create Job
          </Button>
        </Link>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-2">
        {STATUS_FILTERS.map((f) => (
          <button
            key={f.value}
            onClick={() => setStatusFilter(f.value)}
            className={`rounded-full px-3 py-1 text-xs font-medium transition-colors ${
              statusFilter === f.value
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:bg-accent'
            }`}
          >
            {f.label}
          </button>
        ))}
      </div>

      {/* Job list */}
      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-lg bg-muted" />
          ))}
        </div>
      ) : !jobs || jobs.length === 0 ? (
        <div className="rounded-lg border p-12 text-center">
          <Briefcase className="mx-auto h-8 w-8 text-muted-foreground" />
          <p className="mt-2 text-sm text-muted-foreground">No jobs found</p>
        </div>
      ) : (
        <div className="space-y-3">
          {jobs.map((job) => (
            <Link
              key={job.id}
              to={`/jobs/${job.id}`}
              className="block rounded-lg border p-4 transition-colors hover:bg-accent"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <h3 className="font-medium">{job.title}</h3>
                  <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                    {job.description}
                  </p>
                  <div className="mt-2 flex flex-wrap gap-3 text-xs text-muted-foreground">
                    <span>{job.milestone_count} milestones</span>
                    <span>·</span>
                    <span>#{job.id}</span>
                    <span>·</span>
                    <span>{job.client.slice(0, 8)}…</span>
                  </div>
                </div>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${statusColor(job.status)}`}>
                  {job.status}
                </span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
