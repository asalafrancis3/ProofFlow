import { openDB, DBSchema, IDBPDatabase } from 'idb'

interface ScavengerDB extends DBSchema {
  queries: {
    key: string
    value: {
      data: unknown
      timestamp: number
      queryKey: string[]
    }
  }
  mutations: {
    key: string
    value: {
      id: string
      mutationKey: string[]
      variables: unknown
      timestamp: number
      status: 'pending' | 'synced' | 'failed'
      retryCount: number
    }
    indexes: {
      status: 'pending' | 'synced' | 'failed'
      timestamp: number
    }
  }
  cache: {
    key: string
    value: {
      data: unknown
      timestamp: number
      expiresAt?: number
    }
  }
}

let dbPromise: Promise<IDBPDatabase<ScavengerDB>> | null = null

export function getDB() {
  if (!dbPromise) {
    dbPromise = openDB<ScavengerDB>('scavenger-db', 1, {
      upgrade(db) {
        if (!db.objectStoreNames.contains('queries')) {
          db.createObjectStore('queries')
        }

        if (!db.objectStoreNames.contains('mutations')) {
          const mutationsStore = db.createObjectStore('mutations', { keyPath: 'id' })
          mutationsStore.createIndex('status', 'status')
          mutationsStore.createIndex('timestamp', 'timestamp')
        }

        if (!db.objectStoreNames.contains('cache')) {
          db.createObjectStore('cache')
        }
      },
    })
  }
  return dbPromise
}

export async function setQueryData(key: string, data: unknown, queryKey: string[]) {
  const db = await getDB()
  await db.put('queries', {
    data,
    timestamp: Date.now(),
    queryKey,
  }, key)
}

export async function getQueryData(key: string) {
  const db = await getDB()
  return await db.get('queries', key)
}

export async function removeQueryData(key: string) {
  const db = await getDB()
  await db.delete('queries', key)
}

export async function clearExpiredQueries() {
  const db = await getDB()
  const tx = db.transaction('queries', 'readwrite')
  const store = tx.objectStore('queries')

  const now = Date.now()
  const keysToDelete: string[] = []

  for await (const cursor of store) {
    if (now - cursor.value.timestamp > 24 * 60 * 60 * 1000) {
      keysToDelete.push(cursor.key)
    }
  }

  await Promise.all(keysToDelete.map(key => store.delete(key)))
  await tx.done
}

export async function addMutationToQueue(mutation: {
  id: string
  mutationKey: string[]
  variables: unknown
}) {
  const db = await getDB()
  await db.put('mutations', {
    ...mutation,
    timestamp: Date.now(),
    status: 'pending',
    retryCount: 0,
  })
}

export async function getPendingMutations() {
  const db = await getDB()
  const tx = db.transaction('mutations', 'readonly')
  const store = tx.objectStore('mutations')
  const index = store.index('status')

  const mutations = await index.getAll('pending')
  await tx.done

  return mutations.sort((a, b) => a.timestamp - b.timestamp)
}

export async function updateMutationStatus(id: string, status: 'pending' | 'synced' | 'failed', retryCount?: number) {
  const db = await getDB()
  const mutation = await db.get('mutations', id)
  if (mutation) {
    await db.put('mutations', {
      ...mutation,
      status,
      retryCount: retryCount ?? mutation.retryCount,
    })
  }
}

export async function removeMutationFromQueue(id: string) {
  const db = await getDB()
  await db.delete('mutations', id)
}

export async function clearOldMutations() {
  const db = await getDB()
  const tx = db.transaction('mutations', 'readwrite')
  const store = tx.objectStore('mutations')

  const now = Date.now()
  const keysToDelete: string[] = []

  for await (const cursor of store) {
    if (now - cursor.value.timestamp > 7 * 24 * 60 * 60 * 1000) {
      keysToDelete.push(cursor.key)
    }
  }

  await Promise.all(keysToDelete.map(key => store.delete(key)))
  await tx.done
}

export async function setCacheData(key: string, data: unknown, ttl?: number) {
  const db = await getDB()
  await db.put('cache', {
    data,
    timestamp: Date.now(),
    expiresAt: ttl ? Date.now() + ttl : undefined,
  }, key)
}

export async function getCacheData(key: string) {
  const db = await getDB()
  const item = await db.get('cache', key)

  if (!item) return null

  if (item.expiresAt && Date.now() > item.expiresAt) {
    await db.delete('cache', key)
    return null
  }

  return item.data
}

export async function clearExpiredCache() {
  const db = await getDB()
  const tx = db.transaction('cache', 'readwrite')
  const store = tx.objectStore('cache')

  const now = Date.now()
  const keysToDelete: string[] = []

  for await (const cursor of store) {
    if (cursor.value.expiresAt && now > cursor.value.expiresAt) {
      keysToDelete.push(cursor.key)
    }
  }

  await Promise.all(keysToDelete.map(key => store.delete(key)))
  await tx.done
}
