/**
 * MSW request handlers for frontend API tests.
 *
 * These handlers mirror the actual backend API response shapes used by the
 * Scavngr frontend. The ApiClient (src/lib/apiClient.ts) makes REST calls
 * to these endpoints; MSW intercepts them during tests.
 *
 * To add a handler:
 *   1. Inspect the backend endpoint response shape
 *   2. Add an http.get/post/put/patch/delete handler below
 *   3. Use realistic fixture data
 *
 * To override a handler in a single test:
 *   import { http, HttpResponse } from 'msw'
 *   server.use(http.get('/api/wastes', () => HttpResponse.json([])))
 */
import { http, HttpResponse } from 'msw'

// ---------------------------------------------------------------------------
// Waste endpoints
// ---------------------------------------------------------------------------

const MOCK_WASTES = [
  {
    waste_id: '1',
    waste_type: 2,
    weight: '15.5',
    current_owner: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
    latitude: '-1.2921',
    longitude: '36.8219',
    recycled_timestamp: 1700000000,
    is_active: true,
    is_confirmed: false,
    confirmer: ''
  },
  {
    waste_id: '2',
    waste_type: 0,
    weight: '8.2',
    current_owner: 'GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO',
    latitude: '-1.3031',
    longitude: '36.7073',
    recycled_timestamp: 1700100000,
    is_active: true,
    is_confirmed: true,
    confirmer: 'GCITNMB4RRXQHBOPVV42LH2T5NHPD6L5PO23XCJZSKJJBR5Z3GIHKZF'
  }
]

// ---------------------------------------------------------------------------
// Participant endpoints
// ---------------------------------------------------------------------------

const MOCK_PARTICIPANTS = [
  {
    address: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
    role: 'Recycler',
    name: 'Alice',
    latitude: -1.2921,
    longitude: 36.8219,
    registered_at: 1700000000
  },
  {
    address: 'GBKGJTSMPLC54YDKYZPAWKQ4HFSJCLB6PWDX36AFZDLXO3YLQAZFXBO',
    role: 'Collector',
    name: 'Bob',
    latitude: -1.3031,
    longitude: 36.7073,
    registered_at: 1700100000
  }
]

// ---------------------------------------------------------------------------
// Incentive endpoints
// ---------------------------------------------------------------------------

const MOCK_INCENTIVES = [
  {
    id: 1,
    rewarder: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
    waste_type: 2,
    reward_points: 100,
    total_budget: 10000,
    remaining_budget: 9900,
    active: true,
    created_at: 1700000000
  }
]

// ---------------------------------------------------------------------------
// Stats / metrics endpoints
// ---------------------------------------------------------------------------

const MOCK_STATS = {
  address: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
  total_earned: 500,
  materials_submitted: 12,
  transfers_count: 3
}

const MOCK_METRICS = {
  total_wastes_count: 150,
  total_tokens_earned: 25000
}

// ---------------------------------------------------------------------------
// Health endpoint
// ---------------------------------------------------------------------------

const MOCK_HEALTH = {
  status: 'ok',
  uptime: 86400
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

export const handlers = [
  // Waste endpoints
  http.get('*/api/wastes', () => {
    return HttpResponse.json({
      wastes: MOCK_WASTES,
      total: MOCK_WASTES.length,
      limit: 100,
      offset: 0
    })
  }),

  http.get('*/api/wastes/:id', ({ params }) => {
    const waste = MOCK_WASTES.find((w) => w.waste_id === params.id)
    if (!waste) {
      return HttpResponse.json({ error: 'Waste not found' }, { status: 404 })
    }
    return HttpResponse.json(waste)
  }),

  http.post('*/api/wastes', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      { waste_id: '99', ...body, is_active: true, is_confirmed: false },
      { status: 201 }
    )
  }),

  // Participant endpoints
  http.get('*/api/participants', () => {
    return HttpResponse.json({
      participants: MOCK_PARTICIPANTS,
      total: MOCK_PARTICIPANTS.length,
      limit: 100,
      offset: 0
    })
  }),

  http.get('*/api/participants/:address', ({ params }) => {
    const participant = MOCK_PARTICIPANTS.find((p) => p.address === params.address)
    if (!participant) {
      return HttpResponse.json({ error: 'Participant not found' }, { status: 404 })
    }
    return HttpResponse.json(participant)
  }),

  http.post('*/api/participants', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      { ...body, registered_at: Math.floor(Date.now() / 1000) },
      { status: 201 }
    )
  }),

  // Incentive endpoints
  http.get('*/api/incentives', () => {
    return HttpResponse.json({
      incentives: MOCK_INCENTIVES,
      total: MOCK_INCENTIVES.length
    })
  }),

  http.post('*/api/incentives', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      { id: 99, ...body, active: true, created_at: Math.floor(Date.now() / 1000) },
      { status: 201 }
    )
  }),

  // Stats endpoints
  http.get('*/api/stats/:address', ({ params }) => {
    return HttpResponse.json({
      ...MOCK_STATS,
      address: params.address
    })
  }),

  http.get('*/api/metrics', () => {
    return HttpResponse.json(MOCK_METRICS)
  }),

  http.get('*/api/health', () => {
    return HttpResponse.json(MOCK_HEALTH)
  }),
]
