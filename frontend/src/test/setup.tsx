import React from 'react'
import '@testing-library/jest-dom'
import { server } from './msw/server'

// jsdom doesn't provide localStorage. Ensure it exists before any module
// (StoreProvider, auth) tries to access it during initialization.
if (typeof globalThis.localStorage === 'undefined') {
  const store = new Map<string, string>()
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(globalThis as any).localStorage = {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => { store.set(k, String(v)) },
    removeItem: (k: string) => { store.delete(k) },
    clear: () => { store.clear() },
    get length() { return store.size },
    key: (i: number) => [...store.keys()][i] ?? null,
  }
}

// Bridge Node's native fetch to jsdom's window for MSW 2.x
if (typeof globalThis.fetch !== 'undefined' && typeof window !== 'undefined' && !window.fetch) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).fetch = globalThis.fetch.bind(globalThis)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).Request = (globalThis as any).Request
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).Response = (globalThis as any).Response
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).Headers = (globalThis as any).Headers
}

// ── MSW lifecycle ──────────────────────────────────────────────
beforeAll(() => server.listen({ onUnhandledRequest: 'bypass' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

// ── Leaflet stubs (jsdom cannot handle Leaflet DOM APIs) ───────
vi.mock('leaflet', () => ({
  default: {
    divIcon: vi.fn(() => ({})),
    Icon: { Default: { prototype: {}, mergeOptions: vi.fn() } },
    latLngBounds: vi.fn(() => ({ isValid: () => true }))
  },
  divIcon: vi.fn(() => ({}))
}))

vi.mock('react-leaflet', () => ({
  MapContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="map-container">{children}</div>
  ),
  TileLayer: () => null,
  Marker: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="marker">{children}</div>
  ),
  Popup: ({ children }: { children: React.ReactNode }) => <div data-testid="popup">{children}</div>,
  useMap: () => ({ setView: vi.fn(), fitBounds: vi.fn() })
}))

vi.mock('react-leaflet-markercluster', () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="cluster">{children}</div>
  )
}))

vi.mock('leaflet/dist/leaflet.css', () => ({}))
vi.mock('leaflet.markercluster/dist/MarkerCluster.css', () => ({}))
vi.mock('leaflet.markercluster/dist/MarkerCluster.Default.css', () => ({}))
