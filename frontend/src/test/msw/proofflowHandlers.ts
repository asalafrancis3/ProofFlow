import { http, HttpResponse } from 'msw'

const MOCK_JOBS = [
  {
    id: 1,
    client: 'GAAA111111111111111111111111111111111111111111111111',
    title: 'Smart Contract Audit',
    description: 'Audit the ProofFlow contract for security vulnerabilities',
    status: 'active',
    total_funded: 5000,
    milestone_count: 3,
    created_at: 1700000000,
    updated_at: 1700010000,
  },
  {
    id: 2,
    client: 'GAAA222222222222222222222222222222222222222222222222',
    title: 'Frontend Development',
    description: 'Build the ProofFlow dashboard UI',
    status: 'funded',
    total_funded: 3000,
    milestone_count: 2,
    created_at: 1700020000,
    updated_at: 1700030000,
  },
]

const MOCK_REPUTATION = {
  address: 'GAAA111111111111111111111111111111111111111111111111',
  completed_jobs: 5,
  successful_attestations: 12,
  disputes_involved: 1,
  disputes_won: 1,
  total_earned: 15000,
  score: 850,
  updated_at: 1700000000,
}

export const proofflowHandlers = [
  http.get('*/api/v1/health', () => {
    return HttpResponse.json({ success: true, data: { status: 'ok', service: 'proofflow' }, error: null })
  }),

  http.get('*/api/v1/jobs', ({ request }) => {
    const url = new URL(request.url)
    const status = url.searchParams.get('status')
    let filtered = MOCK_JOBS
    if (status) {
      filtered = MOCK_JOBS.filter((j) => j.status === status)
    }
    return HttpResponse.json({ success: true, data: filtered, error: null })
  }),

  http.get('*/api/v1/jobs/:id', ({ params }) => {
    const job = MOCK_JOBS.find((j) => j.id === Number(params.id))
    if (!job) {
      return HttpResponse.json({ success: false, data: null, error: 'Job not found' }, { status: 404 })
    }
    return HttpResponse.json({ success: true, data: job, error: null })
  }),

  http.post('*/api/v1/jobs', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      { success: true, data: { status: 'ok', message: 'Job created', job_title: body.title }, error: null },
      { status: 201 }
    )
  }),

  http.post('*/api/v1/jobs/:id/fund', () => {
    return HttpResponse.json({ success: true, data: { status: 'ok', message: 'Job funded' }, error: null })
  }),

  http.get('*/api/v1/jobs/:id/escrow', ({ params }) => {
    return HttpResponse.json({
      success: true,
      data: { job_id: Number(params.id), total_funded: 5000, total_released: 1000, total_frozen: 0, status: 'funded' },
      error: null,
    })
  }),

  http.post('*/api/v1/jobs/:jobId/milestones/:idx/evidence', async () => {
    return HttpResponse.json({ success: true, data: { status: 'ok', message: 'Evidence submitted' }, error: null })
  }),

  http.post('*/api/v1/jobs/:jobId/milestones/:idx/approve', () => {
    return HttpResponse.json({ success: true, data: { status: 'ok', message: 'Milestone approved' }, error: null })
  }),

  http.post('*/api/v1/jobs/:jobId/milestones/:idx/reject', async () => {
    return HttpResponse.json({ success: true, data: { status: 'ok', message: 'Milestone rejected' }, error: null })
  }),

  http.get('*/api/v1/reputation/:address', ({ params }) => {
    return HttpResponse.json({
      success: true,
      data: { ...MOCK_REPUTATION, address: params.address },
      error: null,
    })
  }),

  http.get('*/api/v1/verifiers', () => {
    return HttpResponse.json({ success: true, data: [], error: null })
  }),

  http.post('*/api/v1/users', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      { success: true, data: { address: body.address, role: body.role, name: body.name, registered_at: Date.now() }, error: null },
      { status: 201 }
    )
  }),

  http.get('*/api/v1/users/:address', ({ params }) => {
    return HttpResponse.json({
      success: true,
      data: { address: params.address, role: 'worker', name: 'Test Worker', registered_at: 1700000000 },
      error: null,
    })
  }),
]
