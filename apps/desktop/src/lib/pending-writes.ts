const pendingWrites = new Set<Promise<unknown>>();
const draftFlushers = new Set<() => Promise<void>>();

export function trackWrite<T>(write: Promise<T>): Promise<T> {
  pendingWrites.add(write);
  void write.finally(() => pendingWrites.delete(write)).catch(() => {});
  return write;
}

export function registerDraftFlusher(flush: () => Promise<void>): () => void {
  draftFlushers.add(flush);
  return () => draftFlushers.delete(flush);
}

export async function flushPendingWrites(): Promise<void> {
  // Flushers may enqueue more writes. Drain until none remain before allowing window close.
  for (const flush of draftFlushers) await flush();
  for (;;) {
    const writes = [...pendingWrites];
    if (!writes.length) return;
    const results = await Promise.allSettled(writes);
    const failed = results.find((result) => result.status === 'rejected');
    if (failed?.status === 'rejected') throw failed.reason;
  }
}
