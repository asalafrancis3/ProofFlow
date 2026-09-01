import { useState, useEffect, useMemo, useRef } from 'react'
import { useProfileStats } from '@/hooks/useProfileStats'
import { useWallet } from '@/context/WalletContext'
import { useNotifications } from '@/hooks/useNotifications'
import {
  evaluateBadges,
  computeLevel,
  generateChallenges,
  buildLeaderboard,
  newlyEarnedBadges,
  Badge,
  Level,
  Challenge,
  LeaderboardEntry,
  UserStats,
} from '@/lib/gamification'

const LS_KEY = (addr: string) => `scavngr_gamification_${addr}`

function saveEarnedIds(addr: string, ids: string[]): void {
  localStorage.setItem(LS_KEY(addr), JSON.stringify(ids))
}

export interface GamificationData {
  badges: Badge[]
  level: Level
  challenges: Challenge[]
  leaderboard: LeaderboardEntry[]
  newBadges: Badge[]
  clearNewBadges: () => void
  isLoading: boolean
}

export function useGamification(): GamificationData {
  const { address } = useWallet()
  const { stats, wastes: _wastes, isLoadingStats } = useProfileStats()
  const { addNotification } = useNotifications(address)
  const [newBadges, setNewBadges] = useState<Badge[]>([])
  const prevStatsRef = useRef<UserStats | null>(null)

  const userStats: UserStats = useMemo(() => ({
    materialsSubmitted: stats?.materials_submitted ?? 0,
    transfersCount: stats?.transfers_count ?? 0,
    totalEarned: stats?.total_earned ?? 0n,
  }), [stats])

  const badges = useMemo(() => evaluateBadges(userStats), [userStats])
  const level = useMemo(() => computeLevel(userStats), [userStats])
  const challenges = useMemo(() => generateChallenges(userStats), [userStats])

  // Leaderboard: current user only (real multi-user data would come from contract)
  const leaderboard = useMemo(
    () => address ? buildLeaderboard([{ address, stats: userStats }], address) : [],
    [address, userStats]
  )

  // Detect newly earned badges and fire notifications
  useEffect(() => {
    if (!address || isLoadingStats) return
    const prev = prevStatsRef.current
    if (!prev) { prevStatsRef.current = userStats; return }

    const earned = newlyEarnedBadges(prev, userStats)
    if (earned.length > 0) {
      setNewBadges(earned)
      saveEarnedIds(address, badges.filter((b) => b.earned).map((b) => b.id))
      earned.forEach((b) => {
        addNotification({
          type: 'reward',
          title: `Badge Unlocked: ${b.name}`,
          body: b.description,
          createdAt: Date.now(),
        })
      })
    }
    prevStatsRef.current = userStats
  }, [userStats, address, isLoadingStats, badges, addNotification])

  return {
    badges,
    level,
    challenges,
    leaderboard,
    newBadges,
    clearNewBadges: () => setNewBadges([]),
    isLoading: isLoadingStats,
  }
}
