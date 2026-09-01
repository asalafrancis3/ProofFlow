/**
 * Offline Data Storage using IndexedDB
 * Provides persistent storage for offline functionality
 */

import { openDB, DBSchema, IDBPDatabase } from 'idb';

interface OfflineDBSchema extends DBSchema {
  queries: {
    key: string;
    value: {
      key: string;
      data: unknown;
      timestamp: number;
    };
  };
  mutations: {
    key: string;
    value: {
      id: string;
      mutationKey: string[];
      variables: unknown;
      timestamp: number;
      retries: number;
      status: 'pending' | 'syncing' | 'failed' | 'success';
      error?: string;
    };
    indexes: {
      status: 'pending' | 'syncing' | 'failed' | 'success';
      timestamp: number;
    };
  };
  cache: {
    key: string;
    value: {
      key: string;
      data: unknown;
      timestamp: number;
      expiresAt?: number;
    };
  };
  settings: {
    key: string;
    value: {
      key: string;
      value: unknown;
    };
  };
}

const DB_NAME = 'scavenger-offline-db';
const DB_VERSION = 1;

let dbPromise: Promise<IDBPDatabase<OfflineDBSchema>> | null = null;

export async function initDB(): Promise<IDBPDatabase<OfflineDBSchema>> {
  if (!dbPromise) {
    dbPromise = openDB<OfflineDBSchema>(DB_NAME, DB_VERSION, {
      upgrade(db) {
        // Create object stores if they don't exist
        if (!db.objectStoreNames.contains('queries')) {
          db.createObjectStore('queries', { keyPath: 'key' });
        }
        if (!db.objectStoreNames.contains('mutations')) {
          const mutationStore = db.createObjectStore('mutations', { keyPath: 'id' });
          mutationStore.createIndex('status', 'status');
          mutationStore.createIndex('timestamp', 'timestamp');
        }
        if (!db.objectStoreNames.contains('cache')) {
          db.createObjectStore('cache', { keyPath: 'key' });
        }
        if (!db.objectStoreNames.contains('settings')) {
          db.createObjectStore('settings', { keyPath: 'key' });
        }
      },
    });
  }
  return dbPromise;
}

// Query Cache Operations
export async function saveQuery(key: string, data: unknown): Promise<void> {
  const db = await initDB();
  await db.put('queries', {
    key,
    data,
    timestamp: Date.now(),
  });
}

export async function getQuery(key: string): Promise<unknown | undefined> {
  const db = await initDB();
  const item = await db.get('queries', key);
  return item?.data;
}

export async function deleteQuery(key: string): Promise<void> {
  const db = await initDB();
  await db.delete('queries', key);
}

export async function clearQueries(): Promise<void> {
  const db = await initDB();
  await db.clear('queries');
}

// Mutation Queue Operations
export interface QueuedMutation {
  id: string;
  mutationKey: string[];
  variables: unknown;
  timestamp: number;
  retries: number;
  status: 'pending' | 'syncing' | 'failed' | 'success';
  error?: string;
}

export async function queueMutation(mutation: Omit<QueuedMutation, 'id' | 'timestamp' | 'retries' | 'status'>): Promise<string> {
  const db = await initDB();
  const id = `mutation-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  
  await db.put('mutations', {
    id,
    ...mutation,
    timestamp: Date.now(),
    retries: 0,
    status: 'pending',
  });
  
  return id;
}

export async function getPendingMutations(): Promise<QueuedMutation[]> {
  const db = await initDB();
  const tx = db.transaction('mutations', 'readonly');
  const index = tx.store.index('status');
  const mutations = await index.getAll('pending');
  await tx.done;
  return mutations;
}

export async function updateMutationStatus(
  id: string,
  status: QueuedMutation['status'],
  error?: string
): Promise<void> {
  const db = await initDB();
  const mutation = await db.get('mutations', id);
  if (mutation) {
    mutation.status = status;
    if (error) mutation.error = error;
    if (status === 'syncing') mutation.retries += 1;
    await db.put('mutations', mutation);
  }
}

export async function deleteMutation(id: string): Promise<void> {
  const db = await initDB();
  await db.delete('mutations', id);
}

export async function clearMutations(): Promise<void> {
  const db = await initDB();
  await db.clear('mutations');
}

// General Cache Operations
export async function setCache(key: string, data: unknown, ttl?: number): Promise<void> {
  const db = await initDB();
  await db.put('cache', {
    key,
    data,
    timestamp: Date.now(),
    expiresAt: ttl ? Date.now() + ttl : undefined,
  });
}

export async function getCache(key: string): Promise<unknown | undefined> {
  const db = await initDB();
  const item = await db.get('cache', key);
  
  if (!item) return undefined;
  
  // Check if expired
  if (item.expiresAt && item.expiresAt < Date.now()) {
    await db.delete('cache', key);
    return undefined;
  }
  
  return item.data;
}

export async function deleteCache(key: string): Promise<void> {
  const db = await initDB();
  await db.delete('cache', key);
}

export async function clearCache(): Promise<void> {
  const db = await initDB();
  await db.clear('cache');
}

// Settings Operations
export async function setSetting(key: string, value: unknown): Promise<void> {
  const db = await initDB();
  await db.put('settings', { key, value });
}

export async function getSetting(key: string): Promise<unknown | undefined> {
  const db = await initDB();
  const item = await db.get('settings', key);
  return item?.value;
}

// Database Management
export async function clearAllData(): Promise<void> {
  const db = await initDB();
  await Promise.all([
    db.clear('queries'),
    db.clear('mutations'),
    db.clear('cache'),
    db.clear('settings'),
  ]);
}

export async function getDatabaseSize(): Promise<{ bytes: number; formatted: string }> {
  if ('storage' in navigator && 'estimate' in navigator.storage) {
    const estimate = await navigator.storage.estimate();
    const bytes = estimate.usage || 0;
    const formatted = formatBytes(bytes);
    return { bytes, formatted };
  }
  return { bytes: 0, formatted: '0 B' };
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}
