/**
 * Sync Manager
 * Handles synchronization of offline data when connection is restored
 */

import {
  getPendingMutations,
  updateMutationStatus,
  deleteMutation,
  QueuedMutation,
} from './storage';

type SyncCallback = (mutation: QueuedMutation) => Promise<unknown>;

export class SyncManager {
  private callbacks: Map<string, SyncCallback> = new Map();
  private isSyncing = false;
  private syncPromise: Promise<SyncResult> | null = null;

  /**
   * Register a mutation handler
   */
  registerMutationHandler(mutationKey: string, callback: SyncCallback): void {
    this.callbacks.set(mutationKey, callback);
  }

  /**
   * Unregister a mutation handler
   */
  unregisterMutationHandler(mutationKey: string): void {
    this.callbacks.delete(mutationKey);
  }

  /**
   * Sync all pending mutations
   */
  async syncPendingMutations(): Promise<SyncResult> {
    if (this.isSyncing) {
      return this.syncPromise as Promise<SyncResult>;
    }

    this.isSyncing = true;
    this.syncPromise = this.performSync();

    try {
      return await this.syncPromise;
    } finally {
      this.isSyncing = false;
      this.syncPromise = null;
    }
  }

  private async performSync(): Promise<SyncResult> {
    const result: SyncResult = {
      total: 0,
      success: 0,
      failed: 0,
      errors: [],
    };

    try {
      const pending = await getPendingMutations();
      result.total = pending.length;

      if (pending.length === 0) {
        return result;
      }

      for (const mutation of pending) {
        try {
          await this.syncSingleMutation(mutation);
          result.success++;
        } catch (error) {
          result.failed++;
          result.errors.push({
            mutationId: mutation.id,
            error: error instanceof Error ? error.message : String(error),
          });
          console.error(`Failed to sync mutation ${mutation.id}:`, error);
        }
      }

    } catch (error) {
      console.error('Sync failed:', error);
      throw error;
    }

    return result;
  }

  private async syncSingleMutation(mutation: QueuedMutation): Promise<void> {
    const mutationKey = mutation.mutationKey.join(':');
    const callback = this.callbacks.get(mutationKey);

    if (!callback) {
      throw new Error(`No handler registered for mutation: ${mutationKey}`);
    }

    // Update status to syncing
    await updateMutationStatus(mutation.id, 'syncing');

    try {
      // Execute the mutation
      await callback(mutation);

      // Mark as success and delete
      await updateMutationStatus(mutation.id, 'success');
      await deleteMutation(mutation.id);
    } catch (error) {
      // Mark as failed
      const errorMessage = error instanceof Error ? error.message : String(error);
      await updateMutationStatus(mutation.id, 'failed', errorMessage);
      throw error;
    }
  }

  /**
   * Check if currently syncing
   */
  get syncing(): boolean {
    return this.isSyncing;
  }
}

export interface SyncResult {
  total: number;
  success: number;
  failed: number;
  errors: Array<{
    mutationId: string;
    error: string;
  }>;
}

// Singleton instance
export const syncManager = new SyncManager();

// Auto-sync when connection is restored
if (typeof window !== 'undefined') {
  window.addEventListener('online', async () => {
    try {
      await syncManager.syncPendingMutations();
    } catch (error) {
      console.error('Auto-sync failed:', error);
    }
  });
}
