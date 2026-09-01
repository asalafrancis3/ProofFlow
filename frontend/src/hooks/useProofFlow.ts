// ProofFlow hooks — React Query hooks for the ProofFlow API

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import * as api from '../api/proofflowClient';
import type { CreateJobRequest, FileDisputeRequest, ResolveDisputeRequest } from '../api/proofflow';

// ── Query keys ──────────────────────────────────────────────────────────────

export const pfKeys = {
  all: ['proofflow'] as const,
  jobs: () => [...pfKeys.all, 'jobs'] as const,
  job: (id: number) => [...pfKeys.jobs(), id] as const,
  escrow: (jobId: number) => [...pfKeys.all, 'escrow', jobId] as const,
  reputation: (address: string) => [...pfKeys.all, 'reputation', address] as const,
  user: (address: string) => [...pfKeys.all, 'user', address] as const,
  verifiers: () => [...pfKeys.all, 'verifiers'] as const,
};

// ── Jobs ────────────────────────────────────────────────────────────────────

export function useJobs(status?: string) {
  return useQuery({
    queryKey: status ? [...pfKeys.jobs(), { status }] : pfKeys.jobs(),
    queryFn: () => api.listJobs(status),
    staleTime: 30_000,
  });
}

export function useJob(jobId: number) {
  return useQuery({
    queryKey: pfKeys.job(jobId),
    queryFn: () => api.getJob(jobId),
    enabled: jobId > 0,
    staleTime: 15_000,
  });
}

export function useCreateJob() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: CreateJobRequest) => api.createJob(req),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: pfKeys.jobs() });
    },
  });
}

// ── Escrow ──────────────────────────────────────────────────────────────────

export function useEscrow(jobId: number) {
  return useQuery({
    queryKey: pfKeys.escrow(jobId),
    queryFn: () => api.getEscrow(jobId),
    enabled: jobId > 0,
    staleTime: 15_000,
  });
}

export function useFundJob() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (jobId: number) => api.fundJob(jobId),
    onSuccess: (_data, jobId) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(jobId) });
      qc.invalidateQueries({ queryKey: pfKeys.escrow(jobId) });
    },
  });
}

// ── Milestones ──────────────────────────────────────────────────────────────

export function useSubmitEvidence() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      jobId,
      milestoneIdx,
      evidenceUri,
      notes,
    }: {
      jobId: number;
      milestoneIdx: number;
      evidenceUri: string;
      notes: string;
    }) => api.submitEvidence(jobId, milestoneIdx, evidenceUri, notes),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(vars.jobId) });
    },
  });
}

export function useApproveMilestone() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ jobId, milestoneIdx }: { jobId: number; milestoneIdx: number }) =>
      api.approveMilestone(jobId, milestoneIdx),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(vars.jobId) });
    },
  });
}

export function useRejectMilestone() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      jobId,
      milestoneIdx,
      reason,
    }: {
      jobId: number;
      milestoneIdx: number;
      reason: string;
    }) => api.rejectMilestone(jobId, milestoneIdx, reason),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(vars.jobId) });
    },
  });
}

// ── Disputes ────────────────────────────────────────────────────────────────

export function useFileDispute() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: FileDisputeRequest) => api.fileDispute(req),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(vars.job_id) });
    },
  });
}

export function useResolveDispute() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: ResolveDisputeRequest) => api.resolveDispute(req),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.job(vars.job_id) });
    },
  });
}

// ── Reputation ──────────────────────────────────────────────────────────────

export function useReputation(address: string) {
  return useQuery({
    queryKey: pfKeys.reputation(address),
    queryFn: () => api.getReputation(address),
    enabled: !!address,
    staleTime: 60_000,
  });
}

// ── User ────────────────────────────────────────────────────────────────────

export function useUser(address: string) {
  return useQuery({
    queryKey: pfKeys.user(address),
    queryFn: () => api.getUser(address),
    enabled: !!address,
    staleTime: 60_000,
  });
}

export function useRegisterUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      address,
      role,
      name,
    }: {
      address: string;
      role: string;
      name: string;
    }) => api.registerUser(address, role, name),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: pfKeys.user(vars.address) });
    },
  });
}

// ── Verifiers ───────────────────────────────────────────────────────────────

export function useVerifiers() {
  return useQuery({
    queryKey: pfKeys.verifiers(),
    queryFn: () => api.listVerifiers(),
    staleTime: 60_000,
  });
}
