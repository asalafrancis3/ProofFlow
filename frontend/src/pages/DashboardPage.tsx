import { Link } from 'react-router-dom'
import { Briefcase, DollarSign, Shield, CheckCircle2, ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { StatCard } from '@/components/ui/StatCard'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useWallet } from '@/context/WalletContext'
import { useJobs, useReputation } from '@/hooks/useProofFlow'

export function DashboardPage() {
  useAppTitle('Dashboard — ProofFlow')
  const { address } = useWallet()
  const { data: jobs, isLoading: jobsLoading } = useJobs()
  const { data: reputation } = useReputation(address ?? '')

  const myJobs = jobs?.filter((j) => j.client === address) ?? []
  const activeJobs = myJobs.filter((j) => j.status === 'active' || j.status === 'funded')
  const settledJobs = myJobs.filter((j) => j.status === 'settled')

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          {address ? `Connected: ${address.slice(0, 8)}...${address.slice(-4)}` : 'Connect your wallet to get started'}
        </p>
      </div>

      {/* Stats */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label="My Jobs"
          value={myJobs.length.toString()}
          icon={<Briefcase className="h-4 w-4" />}
        />
        <StatCard
          label="Active"
          value={activeJobs.length.toString()}
          icon={<DollarSign className="h-4 w-4" />}
        />
        <StatCard
          label="Settled"
          value={settledJobs.length.toString()}
          icon={<CheckCircle2 className="h-4 w-4" />}
        />
        <StatCard
          label="Reputation"
          value={reputation?.score?.toString() ?? '0'}
          icon={<Shield className="h-4 w-4" />}
        />
      </div>

      {/* Quick actions */}
      <div className="grid gap-4 sm:grid-cols-2">
        <Link to="/jobs/new">
          <Button variant="outline" className="w-full justify-between">
            Create New Job
            <ArrowRight className="h-4 w-4" />
          </Button>
        </Link>
        <Link to="/jobs">
          <Button variant="outline" className="w-full justify-between">
            Browse Jobs
            <ArrowRight className="h-4 w-4" />
          </Button>
        </Link>
      </div>

      {/* Recent jobs */}
      <div>
        <h2 className="text-lg font-semibold">Recent Jobs</h2>
        {jobsLoading ? (
          <p className="mt-2 text-sm text-muted-foreground">Loading jobs…</p>
        ) : myJobs.length === 0 ? (
          <div className="mt-4 rounded-lg border p-8 text-center">
            <Briefcase className="mx-auto h-8 w-8 text-muted-foreground" />
            <p className="mt-2 text-sm text-muted-foreground">No jobs yet</p>
            <Link to="/jobs/new">
              <Button size="sm" className="mt-4">
                Create Your First Job
              </Button>
            </Link>
          </div>
        ) : (
          <div className="mt-4 space-y-2">
            {myJobs.slice(0, 5).map((job) => (
              <Link
                key={job.id}
                to={`/jobs/${job.id}`}
                className="flex items-center justify-between rounded-lg border p-4 hover:bg-accent"
              >
                <div>
                  <p className="font-medium">{job.title}</p>
                  <p className="text-xs text-muted-foreground">
                    {job.milestone_count} milestones · {job.status}
                  </p>
                </div>
                <span className="text-xs font-medium text-muted-foreground">
                  #{job.id}
                </span>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
