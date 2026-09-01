/**
 * context/AuthContext.tsx
 *
 * Compatibility shim — delegates to the auth store slice.
 * Existing components that call useAuth() continue to work unchanged.
 *
 * The actual state and dispatch live in StoreProvider (store/index.tsx).
 */
import React, { type ReactNode } from 'react'
import { useAuthStore, type AuthUser } from '@/store'

export type { AuthUser }

// Kept for type consumers that import from this file
export interface AuthContextType {
  user: AuthUser | null
  isAuthenticated: boolean
  login: (user: AuthUser) => void
  logout: () => void
  isLoading: boolean
}

/**
 * AuthProvider is a no-op wrapper — state lives in StoreProvider.
 * Kept so existing JSX (<AuthProvider>) does not need to change.
 */
export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => (
  <>{children}</>
)

/**
 * useAuth — thin adapter over the auth store slice.
 * Returns the same shape as the original AuthContext.
 */
// eslint-disable-next-line react-refresh/only-export-components
export function useAuth(): AuthContextType {
  const { state, dispatch } = useAuthStore()

  return {
    user: state.user,
    isAuthenticated: state.isAuthenticated,
    isLoading: state.isLoading,
    login: (user: AuthUser) => dispatch({ type: 'AUTH_LOGIN', payload: user }),
    logout: () => dispatch({ type: 'AUTH_LOGOUT' }),
  }
}
