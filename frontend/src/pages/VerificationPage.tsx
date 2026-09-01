import { useState } from 'react'
import { Shield, CheckCircle2, XCircle, FileCheck } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useJobs, useApproveMilestone, useRejectMilestone } from '@/hooks/useProofFlow'

export function VerificationPage() {
  useAppTitle('Verification Queue — ProofFlow')
  const { data: jobs, isLoading } = useJobs('in_review')
  const approveMilestone = useApproveMilestone()
  const rejectMilestone = useRejectMilestone()
  const [rejectReason, setRejectReason] = useState('')
  const [selectedJob, setSelectedJob] = useState<number | null>(null)

  const handleApprove = (jobId: number, milestoneIdx: number) => {
    approveMilestone.mutate({ jobId, milestoneIdx })
  }

  const handleReject = (jobId: number, milestoneIdx: number) => {
    if (!rejectReason.trim()) return
    rejectMilestone.mutate({ jobId, milestoneIdx, reason: rejectReason.trim() })
    setRejectReason('')
    setSelectedJob(null)
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Verification Queue</h1>
        <p className="text-sm text-muted-foreground">Review evidence submissions and approve milestones</p>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-lg bg-muted" />
          ))}
        </div>
      ) : !jobs || jobs.length === 0 ? (
        <div className="rounded-lg border p-12 text-center">
          <Shield className="mx-auto h-8 w-8 text-muted-foreground" />
          <p className="mt-2 text-sm text-muted-foreground">No items awaiting verification</p>
        </div>
      ) : (
        <div className="space-y-4">
          {jobs.map((job) => (
            <div key={job.id} className="rounded-lg border p-4">
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-medium">{job.title}</h3>
                  <p className="text-xs text-muted-foreground">Job #{job.id} · {job.milestone_count} milestones</p>
                </div>
                <span className="rounded-full bg-yellow-100 px-2 py-0.5 text-xs font-medium text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300">
                  In Review
                </span>
              </div>

              {/* Milestones to review */}
              <div className="mt-4 space-y-2">
                {Array.from({ length: job.milestone_count }, (_, i) => (
                  <div key={i} className="flex items-center justify-between rounded-md bg-muted/50 p-3">
                    <div className="flex items-center gap-2">
                      <FileCheck className="h-4 w-4 text-muted-foreground" />
                      <span className="text-sm">Milestone {i + 1}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleApprove(job.id, i)}
                        disabled={approveMilestone.isPending}
                      >
                        <CheckCircle2 className="mr-1 h-3 w-3" /> Approve
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => {
                          setSelectedJob(selectedJob === job.id ? null : job.id)
                        }}
                      >
                        <XCircle className="mr-1 h-3 w-3" /> Reject
                      </Button>
                    </div>
                  </div>
                ))}
              </div>

              {/* Reject reason (shown when this job is selected) */}
              {selectedJob === job.id && (
                <div className="mt-3 flex gap-2">
                  <Input
                    value={rejectReason}
                    onChange={(e) => setRejectReason(e.target.value)}
                    placeholder="Reason for rejection..."
                    className="flex-1"
                  />
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => handleReject(job.id, 0)}
                    disabled={!rejectReason.trim() || rejectMilestone.isPending}
                  >
                    Confirm Reject
                  </Button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
