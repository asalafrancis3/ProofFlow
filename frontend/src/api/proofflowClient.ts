// ProofFlow API client — wraps the backend REST API

import type {
  ApiResponse,
  CreateJobRequest,
  Dispute,
  Escrow,
  FileDisputeRequest,
  Job,
  Reputation,
  ResolveDisputeRequest,
  User,
} from './proofflow';

const API_BASE = import.meta.env.VITE_API_URL ?? '/api/v1';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  const body: ApiResponse<T> = await res.json();
  if (!body.success) {
    throw new Error(body.error ?? 'Unknown API error');
  }
  return body.data as T;
}

// ── Health ──────────────────────────────────────────────────────────────────

export async function healthCheck(): Promise<{ status: string; service: string }> {
  return request('/health');
}

// ── Users ───────────────────────────────────────────────────────────────────

export async function registerUser(address: string, role: string, name: string): Promise<User> {
  return request('/users', {
    method: 'POST',
    body: JSON.stringify({ address, role, name }),
  });
}

export async function getUser(address: string): Promise<User> {
  return request(`/users/${address}`);
}

// ── Jobs ────────────────────────────────────────────────────────────────────

export async function createJob(req: CreateJobRequest): Promise<{ status: string; message: string; job_title: string }> {
  return request('/jobs', {
    method: 'POST',
    body: JSON.stringify(req),
  });
}

export async function getJob(jobId: number): Promise<Job> {
  return request(`/jobs/${jobId}`);
}

export async function listJobs(status?: string): Promise<Job[]> {
  const params = status && status !== 'all' ? `?status=${status}` : '';
  return request(`/jobs${params}`);
}

// ── Escrow ──────────────────────────────────────────────────────────────────

export async function fundJob(jobId: number): Promise<{ status: string; message: string }> {
  return request(`/jobs/${jobId}/fund`, { method: 'POST' });
}

export async function getEscrow(jobId: number): Promise<Escrow> {
  return request(`/jobs/${jobId}/escrow`);
}

// ── Milestones ──────────────────────────────────────────────────────────────

export async function submitEvidence(
  jobId: number,
  milestoneIdx: number,
  evidenceUri: string,
  notes: string,
): Promise<{ status: string; message: string }> {
  return request(`/jobs/${jobId}/milestones/${milestoneIdx}/evidence`, {
    method: 'POST',
    body: JSON.stringify({ evidence_uri: evidenceUri, notes }),
  });
}

export async function approveMilestone(
  jobId: number,
  milestoneIdx: number,
): Promise<{ status: string; message: string }> {
  return request(`/jobs/${jobId}/milestones/${milestoneIdx}/approve`, { method: 'POST' });
}

export async function rejectMilestone(
  jobId: number,
  milestoneIdx: number,
  reason: string,
): Promise<{ status: string; message: string }> {
  return request(`/jobs/${jobId}/milestones/${milestoneIdx}/reject`, {
    method: 'POST',
    body: JSON.stringify({ reason }),
  });
}

// ── Disputes ────────────────────────────────────────────────────────────────

export async function fileDispute(req: FileDisputeRequest): Promise<{ status: string; message: string }> {
  return request('/disputes', {
    method: 'POST',
    body: JSON.stringify(req),
  });
}

export async function getDispute(jobId: number, disputeId: number): Promise<Dispute> {
  return request(`/disputes/${jobId}/${disputeId}`);
}

export async function resolveDispute(req: ResolveDisputeRequest): Promise<{ status: string; message: string }> {
  return request('/disputes/resolve', {
    method: 'POST',
    body: JSON.stringify(req),
  });
}

// ── Reputation ──────────────────────────────────────────────────────────────

export async function getReputation(address: string): Promise<Reputation> {
  return request(`/reputation/${address}`);
}

// ── Verifiers ───────────────────────────────────────────────────────────────

export async function listVerifiers(): Promise<string[]> {
  return request('/verifiers');
}
