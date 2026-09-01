import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { ArrowLeft, Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { useAppTitle } from '@/hooks/useAppTitle'
import { useCreateJob } from '@/hooks/useProofFlow'
import { useWallet } from '@/context/WalletContext'

interface MilestoneInput {
  title: string
  amount: string
  worker: string
}

export function CreateJobPage() {
  useAppTitle('Create Job — ProofFlow')
  const navigate = useNavigate()
  const { address } = useWallet()
  const createJob = useCreateJob()

  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [milestones, setMilestones] = useState<MilestoneInput[]>([
    { title: '', amount: '', worker: '' },
  ])

  const addMilestone = () => {
    setMilestones([...milestones, { title: '', amount: '', worker: '' }])
  }

  const removeMilestone = (idx: number) => {
    if (milestones.length > 1) {
      setMilestones(milestones.filter((_, i) => i !== idx))
    }
  }

  const updateMilestone = (idx: number, field: keyof MilestoneInput, value: string) => {
    const updated = [...milestones]
    updated[idx] = { ...updated[idx], [field]: value }
    setMilestones(updated)
  }

  const isValid =
    title.trim() &&
    description.trim() &&
    milestones.every((m) => m.title.trim() && m.amount && Number(m.amount) > 0)

  const handleSubmit = async () => {
    if (!address || !isValid) return
    try {
      await createJob.mutateAsync({
        client: address,
        title: title.trim(),
        description: description.trim(),
        milestone_titles: milestones.map((m) => m.title.trim()),
        milestone_amounts: milestones.map((m) => Number(m.amount)),
        milestone_workers: milestones.map((m) => m.worker.trim()),
      })
      navigate('/jobs')
    } catch {
      // Error handled by useToast
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <Link to="/jobs" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="h-4 w-4" /> Back to Jobs
      </Link>

      <div>
        <h1 className="text-2xl font-bold">Create Job</h1>
        <p className="text-sm text-muted-foreground">Define work, milestones, and payment terms</p>
      </div>

      {/* Job details */}
      <div className="space-y-4 rounded-lg border p-4">
        <h2 className="text-sm font-semibold uppercase text-muted-foreground">Job Details</h2>
        <div>
          <label className="text-sm font-medium">Title</label>
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="e.g., Build a landing page"
            className="mt-1"
          />
        </div>
        <div>
          <label className="text-sm font-medium">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe the work to be done..."
            className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            rows={4}
          />
        </div>
      </div>

      {/* Milestones */}
      <div className="space-y-4 rounded-lg border p-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold uppercase text-muted-foreground">Milestones</h2>
          <Button variant="outline" size="sm" onClick={addMilestone}>
            <Plus className="mr-1 h-3 w-3" /> Add
          </Button>
        </div>

        {milestones.map((m, idx) => (
          <div key={idx} className="space-y-3 rounded-md border p-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-muted-foreground">Milestone {idx + 1}</span>
              {milestones.length > 1 && (
                <button
                  onClick={() => removeMilestone(idx)}
                  className="text-destructive hover:text-destructive/80"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              )}
            </div>
            <Input
              value={m.title}
              onChange={(e) => updateMilestone(idx, 'title', e.target.value)}
              placeholder="Milestone title"
            />
            <div className="grid grid-cols-2 gap-3">
              <Input
                type="number"
                value={m.amount}
                onChange={(e) => updateMilestone(idx, 'amount', e.target.value)}
                placeholder="Amount (tokens)"
                min="1"
              />
              <Input
                value={m.worker}
                onChange={(e) => updateMilestone(idx, 'worker', e.target.value)}
                placeholder="Worker address (optional)"
              />
            </div>
          </div>
        ))}
      </div>

      {/* Submit */}
      <Button
        className="w-full"
        size="lg"
        onClick={handleSubmit}
        disabled={!isValid || createJob.isPending || !address}
      >
        {createJob.isPending ? 'Creating…' : 'Create Job'}
      </Button>

      {!address && (
        <p className="text-center text-sm text-muted-foreground">
          Connect your wallet to create a job
        </p>
      )}
    </div>
  )
}
