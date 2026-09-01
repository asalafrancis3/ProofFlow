// ProofFlow API types — mirrors the backend contract types

export type UserRole = 'client' | 'worker' | 'verifier' | 'arbitrator' | 'admin';

export type JobStatus = 'draft' | 'funded' | 'active' | 'in_review' | 'settled' | 'cancelled' | 'disputed';

export type MilestoneStatus = 'pending' | 'submitted' | 'approved' | 'rejected' | 'released' | 'disputed';

export type EscrowStatus = 'created' | 'funded' | 'partial_release' | 'completed' | 'frozen';

export type DisputeStatus = 'filed' | 'under_review' | 'resolved';

export type Resolution = 'uphold_worker' | 'uphold_client' | 'partial_split';

export interface User {
  address: string;
  role: UserRole;
  name: string;
  registered_at: number;
}

export interface Job {
  id: number;
  client: string;
  title: string;
  description: string;
  status: JobStatus;
  total_funded: number;
  milestone_count: number;
  created_at: number;
  updated_at: number;
}

export interface Milestone {
  job_id: number;
  index: number;
  title: string;
  description: string;
  amount: number;
  status: MilestoneStatus;
  worker: string;
  evidence_uri: string;
  has_evidence: boolean;
  submitted_at: number;
  resolved_at: number;
}

export interface Escrow {
  job_id: number;
  total_funded: number;
  total_released: number;
  total_frozen: number;
  status: EscrowStatus;
}

export interface Dispute {
  job_id: number;
  milestone_idx: number;
  dispute_id: number;
  raised_by: string;
  reason: string;
  status: DisputeStatus;
  resolution: Resolution;
  has_resolution: boolean;
  created_at: number;
  resolved_at: number;
}

export interface Reputation {
  address: string;
  completed_jobs: number;
  successful_attestations: number;
  disputes_involved: number;
  disputes_won: number;
  total_earned: number;
  score: number;
  updated_at: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export interface CreateJobRequest {
  client: string;
  title: string;
  description: string;
  milestone_titles: string[];
  milestone_amounts: number[];
  milestone_workers: string[];
}

export interface SubmitEvidenceRequest {
  worker: string;
  job_id: number;
  milestone_idx: number;
  evidence_uri: string;
  notes: string;
}

export interface FileDisputeRequest {
  raised_by: string;
  job_id: number;
  milestone_idx: number;
  reason: string;
}

export interface ResolveDisputeRequest {
  arbitrator: string;
  job_id: number;
  dispute_id: number;
  resolution: string;
  note: string;
}

// API error codes
export type ErrorCode =
  | 'UNAUTHORIZED'
  | 'ADMIN_REQUIRED'
  | 'USER_NOT_FOUND'
  | 'JOB_NOT_FOUND'
  | 'MILESTONE_NOT_FOUND'
  | 'INSUFFICIENT_FUNDS'
  | 'INSUFFICIENT_ESCROW'
  | 'DISPUTE_NOT_FOUND'
  | 'VALIDATION_FAILED'
  | 'RPC_ERROR'
  | 'INTERNAL_ERROR';
