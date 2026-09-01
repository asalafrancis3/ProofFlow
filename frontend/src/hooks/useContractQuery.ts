/**
 * Thin wrappers around TanStack Query that wire the ScavengerClient into
 * a consistent query/mutation pattern used across the app.
 *
 * useContractQuery  — read-only contract calls (useQuery)
 * useContractMutation — mutating contract calls (useMutation)
 */
import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
  type UseMutationOptions,
  type QueryKey,
} from '@tanstack/react-query'
import { useApi } from './useApi'
import { ScavengerClient } from '@/api/client'
import { ContractError } from '@/api/types'

// ── Query ─────────────────────────────────────────────────────────────────────

type ContractQueryFn<T> = (client: ScavengerClient) => Promise<T>

export function useContractQuery<T>(
  queryKey: QueryKey,
  fn: ContractQueryFn<T>,
  options?: Omit<UseQueryOptions<T, ContractError>, 'queryKey' | 'queryFn'>
) {
  const api = useApi()
  return useQuery<T, ContractError>({
    queryKey,
    queryFn: () => fn(api),
    ...options,
  })
}

// ── Mutation ──────────────────────────────────────────────────────────────────

type ContractMutationFn<TVariables, TData> = (
  client: ScavengerClient,
  variables: TVariables
) => Promise<TData>

export function useContractMutation<TVariables, TData = void>(
  fn: ContractMutationFn<TVariables, TData>,
  options?: Omit<UseMutationOptions<TData, ContractError, TVariables>, 'mutationFn'> & {
    invalidateKeys?: QueryKey[]
  }
) {
  const api = useApi()
  const queryClient = useQueryClient()
  const { invalidateKeys, ...restOptions } = options ?? {}

  return useMutation<TData, ContractError, TVariables>({
    mutationFn: (variables) => fn(api, variables),
    onSuccess: async (data, variables, context) => {
      if (invalidateKeys) {
        await Promise.all(
          invalidateKeys.map((key) => queryClient.invalidateQueries({ queryKey: key }))
        )
      }
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      ;(restOptions as any).onSuccess?.(data, variables, context)
    },
    ...restOptions,
  })
}
