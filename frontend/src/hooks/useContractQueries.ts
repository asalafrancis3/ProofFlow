/**
 * Generic contract query hooks for parameter-based queries.
 *
 * These hooks are used when you need to query data for a specific entity (by ID/address).
 * For hooks that use context (e.g., current wallet), see individual hook files instead.
 *
 * Example:
 *   - useParticipantStats(address) — query stats for a specific address
 *   - useParticipant() from useParticipant.ts — query current wallet's participant
 *
 * See Issue #1056 for the consolidation effort.
 */

import { useQuery } from '@tanstack/react-query'
import { ScavengerClient } from '@/api/client'
import { WasteType } from '@/api/types'
import { useContract } from '@/context/ContractContext'
import { getNetworkPassphrase } from '@/lib/stellar'
import { cacheKeys } from '@/lib/cacheKeys'

function useClient() {
  const { config } = useContract()
  return new ScavengerClient({
    contractId: config.contractId,
    rpcUrl: config.rpcUrl,
    networkPassphrase: getNetworkPassphrase(config.network),
  })
}

/**
 * Query participant data by address.
 * For getting current wallet's participant, use useParticipant() from useParticipant.ts instead.
 */
export function useParticipantById(address: string | undefined) {
  const client = useClient()
  return useQuery({
    queryKey: cacheKeys.participant(address ?? ''),
    queryFn: () => client.getParticipant(address!),
    enabled: !!address,
    staleTime: 60_000,
  })
}

/**
 * Deprecated: Use useParticipantStats from useParticipantStats.ts instead.
 * Kept for backward compatibility.
 */
export function useParticipant(address: string | undefined) {
  return useParticipantById(address)
}

export { useParticipantStats } from '@/hooks/useParticipantStats'

/**
 * Query waste data by ID.
 * For getting current participant's wastes, use useWastes() from useWastes.ts instead.
 */
export function useWaste(id: bigint | undefined) {
  const client = useClient()
  return useQuery({
    queryKey: cacheKeys.waste(id?.toString() ?? ''),
    queryFn: () => client.getWaste(id!),
    enabled: id !== undefined,
    staleTime: 30_000,
  })
}

export { useMetrics } from '@/hooks/useMetrics'
export { useActiveIncentives } from '@/hooks/useActiveIncentives'

/**
 * Query incentives by waste type.
 * For getting all incentives, use useActiveIncentives() instead.
 */
export function useIncentives(wasteType: WasteType | undefined) {
  const client = useClient()
  return useQuery({
    queryKey: cacheKeys.incentives(wasteType !== undefined ? String(wasteType) : undefined),
    queryFn: () => client.getIncentives(wasteType!),
    enabled: wasteType !== undefined,
    staleTime: 5 * 60_000,
  })
}

export { useSupplyChainStats } from '@/hooks/useSupplyChainStats'
