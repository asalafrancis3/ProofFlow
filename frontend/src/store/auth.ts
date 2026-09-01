/**
 * store/auth.ts — Auth slice
 *
 * Manages wallet-based authentication state:
 *   address, name, role, and session loading.
 *
 * Persisted to localStorage under 'proofflow_user'.
 */

export interface AuthUser {
  address: string
  role?: string
  name?: string
}

export interface AuthState {
  user: AuthUser | null
  isAuthenticated: boolean
  isLoading: boolean
}

export type AuthAction =
  | { type: 'AUTH_LOADING' }
  | { type: 'AUTH_LOGIN'; payload: AuthUser }
  | { type: 'AUTH_LOGOUT' }

const STORAGE_KEY = 'proofflow_user'

export const authInitialState: AuthState = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
}

export function authReducer(state: AuthState, action: AuthAction): AuthState {
  switch (action.type) {
    case 'AUTH_LOADING':
      return { ...state, isLoading: true }

    case 'AUTH_LOGIN':
      return {
        user: action.payload,
        isAuthenticated: true,
        isLoading: false,
      }

    case 'AUTH_LOGOUT':
      return {
        user: null,
        isAuthenticated: false,
        isLoading: false,
      }

    default:
      return state
  }
}

/* ── Persistence helpers ─────────────────────────────────────── */

export function persistAuth(user: AuthUser): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(user))
}

export function clearPersistedAuth(): void {
  localStorage.removeItem(STORAGE_KEY)
}

export function loadPersistedAuth(): AuthUser | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? (JSON.parse(raw) as AuthUser) : null
  } catch {
    return null
  }
}
