/**
 * useOfflineMutation Hook
 * Queues mutations when offline and syncs when online
 */

import { useMutation, UseMutationOptions, UseMutationResult } from '@tanstack/react-query';
import { useOnlineStatus } from './useOnlineStatus';
import { queueMutation } from '../lib/offline/storage';

interface OfflineMutationOptions<TData, TError, TVariables>
  extends UseMutationOptions<TData, TError, TVariables> {
  offlineMessage?: string;
}

export function useOfflineMutation<TData = unknown, TError = unknown, TVariables = unknown>(
  options: OfflineMutationOptions<TData, TError, TVariables>
): UseMutationResult<TData, TError, TVariables> & { isOnline: boolean } {
  const isOnline = useOnlineStatus();

  const mutation = useMutation<TData, TError, TVariables>({
    ...options,
    mutationFn: async (variables: TVariables) => {
      if (!isOnline) {
        // Queue mutation for later
        const mutationKey = options.mutationKey || ['offline-mutation'];
        await queueMutation({
          mutationKey: mutationKey as string[],
          variables,
        });

        // Return a placeholder response
        throw new Error(options.offlineMessage || 'Operation queued for when you\'re online');
      }

      // Execute mutation normally when online
      return options.mutationFn!(variables, undefined as never);
    },
  });

  return {
    ...mutation,
    isOnline,
  };
}
