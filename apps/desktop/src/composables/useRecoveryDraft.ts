import { watchDebounced } from '@vueuse/core';
import { onScopeDispose, shallowRef, toValue, watch } from 'vue';
import type { MaybeRefOrGetter } from 'vue';
import { discardDraft, loadDraft, saveDraft } from '../lib/draft-storage';
import { notifications } from '../lib/notifications';
import { registerDraftFlusher, trackWrite } from '../lib/pending-writes';
import { usesNativeStorage } from '../lib/workspace-storage';

export function useRecoveryDraft<T>(key: MaybeRefOrGetter<string>) {
  const value = shallowRef<T | null>(null);
  const error = shallowRef('');
  let version: number | null = null;
  let serial = Promise.resolve();
  let ready = Promise.resolve();
  let generation = 0;
  let revision = 0;
  let loading = false;

  watch(
    () => toValue(key),
    (draftKey) => {
      // Ignore a load from the previous issue or workspace if the key changes mid-request.
      const currentGeneration = ++generation;
      revision++;
      loading = true;
      value.value = null;
      version = null;
      error.value = '';
      ready = (async () => {
        const discardedVersion = localStorage.getItem(`draft-discard:${draftKey}`);
        const legacy = localStorage.getItem(draftKey);
        let parsed: T | null = null;

        try {
          parsed = legacy ? (JSON.parse(legacy) as T) : null;
        } catch {
          localStorage.removeItem(draftKey);
        }

        value.value = parsed;
        const record = await loadDraft<T>(draftKey);
        if (generation !== currentGeneration) return;

        version = record?.version ?? null;
        if (discardedVersion !== null && record?.version === Number(discardedVersion)) {
          value.value = null;
          try {
            if (usesNativeStorage) await discardDraft(draftKey, record.version);
            version = null;
            localStorage.removeItem(`draft-discard:${draftKey}`);
          } catch (cause) {
            error.value = String(cause);
          }

          localStorage.removeItem(draftKey);
          return;
        }

        localStorage.removeItem(`draft-discard:${draftKey}`);
        value.value = record?.payload ?? parsed;
        if (parsed && !record && usesNativeStorage) {
          const migrated = await saveDraft(draftKey, parsed, null);
          version = migrated.version;
        }

        if (record || parsed) localStorage.removeItem(draftKey);
      })()
        .catch((cause) => {
          error.value = String(cause);
        })
        .finally(() => {
          if (generation === currentGeneration) loading = false;
        });
    },
    { immediate: true },
  );

  watchDebounced(
    value,
    (payload) => {
      if (loading) return;
      const draftKey = toValue(key);
      const currentGeneration = generation;
      const currentRevision = revision;
      serial = serial
        .then(async () => {
          // A queued write can outlive a newer edit or a switch to another draft key.
          if (
            generation !== currentGeneration ||
            revision !== currentRevision ||
            value.value !== payload
          )
            return;
          if (payload === null) {
            if (usesNativeStorage) await discardDraft(draftKey, version);
            version = null;
            localStorage.removeItem(draftKey);
          } else {
            localStorage.setItem(draftKey, JSON.stringify(payload));
            if (usesNativeStorage) {
              const saved = await saveDraft(draftKey, payload, version);
              version = saved.version;
              localStorage.removeItem(draftKey);
            }
          }
        })
        .catch((cause) => {
          error.value = String(cause);
        });
    },
    { debounce: 250, deep: true },
  );

  // Keep a synchronous recovery copy if a window closes before native debounce runs.
  watch(
    value,
    (payload) => {
      if (loading) return;
      const draftKey = toValue(key);
      if (payload === null) localStorage.removeItem(draftKey);
      else localStorage.setItem(draftKey, JSON.stringify(payload));
    },
    { deep: true, flush: 'sync' },
  );

  const unregisterFlusher = registerDraftFlusher(flush);

  onScopeDispose(() => {
    unregisterFlusher();
    if (!loading && value.value !== null && usesNativeStorage) {
      const draftKey = toValue(key);
      const payload = value.value;
      void trackWrite(
        serial.then(async () => {
          const saved = await saveDraft(draftKey, payload, version);
          version = saved.version;
          localStorage.removeItem(draftKey);
        }),
      );
    }
  });

  async function waitUntilReady() {
    await ready;
  }

  async function flush() {
    await ready;
    await serial;
    if (loading || value.value === null) return;

    const draftKey = toValue(key);
    const payload = value.value;
    localStorage.setItem(draftKey, JSON.stringify(payload));
    const saved = await saveDraft(draftKey, payload, version);
    version = saved.version;
    localStorage.removeItem(draftKey);
    error.value = '';
  }

  async function discard() {
    // Persist a tombstone until native deletion succeeds; otherwise a failed delete could
    // bring the discarded draft back on the next open.
    revision++;
    const draftKey = toValue(key);
    value.value = null;
    localStorage.removeItem(draftKey);
    serial = serial
      .catch(() => {})
      .then(async () => {
        if (usesNativeStorage && version !== null) {
          localStorage.setItem(`draft-discard:${draftKey}`, String(version));
          await discardDraft(draftKey, version);
        }

        version = null;
        localStorage.removeItem(`draft-discard:${draftKey}`);
      })
      .catch((cause) => {
        error.value = String(cause);
        notifications.add({
          title: 'Draft cleanup pending',
          description: 'Draft cleanup will retry when this screen opens again.',
          type: 'warning',
        });
      });
    await serial;
  }

  return { value, error, waitUntilReady, flush, discard };
}
