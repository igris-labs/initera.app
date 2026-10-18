<script setup lang="ts">
import { usePreferredDark } from '@vueuse/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { MotionConfig } from 'motion-v';
import { storeToRefs } from 'pinia';
import { computed, nextTick, onMounted, onUnmounted, ref, watch, watchEffect } from 'vue';
import AppShell from './components/common/AppShell.vue';
import AppTitlebar from './components/common/AppTitlebar.vue';
import AppUpdater from './components/common/AppUpdater.vue';
import WorkspaceSetup from './components/common/WorkspaceSetup.vue';
import { Toast } from './components/ui/toast';
import { Button } from './components/ui/button';
import { Dialog } from './components/ui/dialog';
import { runRestoreCatchup } from './lib/backup-storage';
import { checkForUpdates } from './lib/app-updater';
import { invoke } from './lib/native-invoke';
import { flushPendingWrites } from './lib/pending-writes';
import { notifications, notifySuccess } from './lib/notifications';
import { usesNativeStorage } from './lib/workspace-storage';
import { usePreferencesStore } from './stores/preferences';
import { useIssuesStore } from './stores/issues';
import { useLabelsStore } from './stores/labels';
import { useProjectsStore } from './stores/projects';
import { useViewsStore } from './stores/views';
import { useWorkspaceStore } from './stores/workspace';

const preferences = usePreferencesStore();
const issues = useIssuesStore();
const labels = useLabelsStore();
const projects = useProjectsStore();
const views = useViewsStore();
const workspace = useWorkspaceStore();
const { theme } = storeToRefs(preferences);
const prefersDark = usePreferredDark();

const mounted = ref(false);
const restoreReviewOpen = ref(false);
const catchupBusy = ref(false);
const catchupError = ref('');
const catchupProgress = ref('');
let unlistenTimerSleep: (() => void) | undefined;
let unlistenTimerReview: (() => void) | undefined;
let unlistenClose: (() => void) | undefined;
const closeRequested = ref(false);
const closeError = ref('');
const closing = ref(false);

interface TimerRecoveryNotice {
  id: string;
  issueIdentifier: string;
  kind: 'closed' | 'long_gap' | 'backward_clock' | 'sleep_uncertain';
  gapSeconds: number;
  gapMs: number;
  createdAt: number;
}

const timerNotices = ref<TimerRecoveryNotice[]>([]);
const timerReviewBusy = ref(false);
const timerReviewError = ref('');
const currentTimerNotice = computed(() => timerNotices.value[0]);

async function loadTimerNotices() {
  if (!usesNativeStorage || !workspace.configured) return;
  timerNotices.value = await invoke<TimerRecoveryNotice[]>('storage_list_timer_recovery_notices');
}

async function resolveTimerNotice(includeGap = false) {
  const notice = currentTimerNotice.value;
  if (!notice) return;
  timerReviewBusy.value = true;
  timerReviewError.value = '';
  try {
    await invoke('storage_resolve_timer_recovery_notice', {
      id: notice.id,
      includeGap,
      mutation: { mutationId: crypto.randomUUID() },
    });
    await loadTimerNotices();
    window.dispatchEvent(new Event('timer-changed'));
  } catch (error) {
    timerReviewError.value = String(error);
  } finally {
    timerReviewBusy.value = false;
  }
}

async function finishClose(force = false) {
  // Native close is deferred until editor flushers settle; a failed save keeps the window open.
  if (closing.value) return;
  closing.value = true;
  closeError.value = '';
  try {
    if (!force) await flushPendingWrites();
    await invoke('storage_finish_close', { force });
    await getCurrentWindow().close();
  } catch (error) {
    closeError.value = String(error);
    closeRequested.value = true;
  } finally {
    closing.value = false;
  }
}

function keepRestoreReviewFocused(event: KeyboardEvent) {
  if (workspace.restoredBackup && event.key === 'Escape') {
    event.preventDefault();
    event.stopImmediatePropagation();
  }
}

const isDark = computed(
  () => theme.value === 'dark' || (theme.value === 'system' && prefersDark.value),
);

watchEffect(() => document.documentElement.classList.toggle('dark', isDark.value));

onMounted(() => {
  mounted.value = true;
  window.addEventListener('keydown', keepRestoreReviewFocused, true);
  workspace.bootstrap();
  if (usesNativeStorage)
    void listen('timer-recovery-changed', () => {
      void loadTimerNotices().catch((error) => {
        timerReviewError.value = String(error);
      });
      window.dispatchEvent(new Event('timer-changed'));
    }).then((unlisten) => {
      unlistenTimerReview = unlisten;
    });
  if (usesNativeStorage)
    void listen('app-close-requested', () => {
      closeRequested.value = true;
      void finishClose();
    }).then((unlisten) => {
      unlistenClose = unlisten;
    });
  if (usesNativeStorage)
    void listen<number>('timers-paused-after-sleep', (event) => {
      notifications.add({
        title: event.payload === 1 ? 'Timer paused after sleep' : 'Timers paused after sleep',
        description: 'Tracked time ends at the last recorded heartbeat.',
        type: 'warning',
      });
      void loadTimerNotices().catch((error) => {
        timerReviewError.value = String(error);
      });
      window.dispatchEvent(new Event('timer-changed'));
    }).then((unlisten) => {
      unlistenTimerSleep = unlisten;
    });
});

onUnmounted(() => {
  window.removeEventListener('keydown', keepRestoreReviewFocused, true);
  unlistenTimerSleep?.();
  unlistenTimerReview?.();
  unlistenClose?.();
});

watch(
  () => [workspace.ready, workspace.workspaceId] as const,
  () => {
    void loadTimerNotices().catch((error) => {
      timerReviewError.value = String(error);
    });
  },
  { immediate: true },
);

watch(
  () => workspace.restoreError,
  (error) => {
    if (error)
      notifications.add({ title: 'Backup restore failed', description: error, type: 'error' });
  },
);

watch(
  () => workspace.restoredBackup,
  (restored) => {
    restoreReviewOpen.value = restored;
  },
);

async function completeRestore() {
  catchupBusy.value = true;
  catchupError.value = '';
  try {
    for (;;) {
      const progress = await runRestoreCatchup();
      catchupProgress.value = progress.done
        ? ''
        : `${progress.processed} of ${progress.total} elapsed cycle periods processed`;
      if (progress.done) break;
      await nextTick();
    }

    restoreReviewOpen.value = false;
    workspace.restoredBackup = false;
    notifySuccess('Workspace restored', 'Cycle catch-up completed.');
  } catch (error) {
    catchupError.value = String(error);
  } finally {
    catchupBusy.value = false;
  }
}

function keepRestoreReviewOpen(open: boolean) {
  if (open || !workspace.restoredBackup) restoreReviewOpen.value = open;
}

watch(
  () => workspace.recoveredTimers,
  (count) => {
    if (count)
      notifications.add({
        title: count === 1 ? 'Timer paused after interruption' : 'Timers paused after interruption',
        description: 'Tracked time ends at the last recorded heartbeat.',
        type: 'warning',
      });
  },
);

const workspaceViewReady = computed(
  () =>
    mounted.value &&
    (Boolean(
      workspace.error ||
      preferences.loadError ||
      issues.loadError ||
      labels.error ||
      projects.loadError ||
      views.loadError,
    ) ||
      (workspace.ready &&
        workspace.configured &&
        preferences.ready &&
        issues.ready &&
        labels.ready &&
        projects.ready &&
        views.ready)),
);

let checkedForUpdates = false;

watch(workspaceViewReady, (ready) => {
  if (!ready || !workspace.configured || checkedForUpdates || !import.meta.env.PROD) return;
  checkedForUpdates = true;
  void checkForUpdates();
});

watch(workspaceViewReady, async (ready) => {
  // Keep the transition visible through data bootstrap and the first painted workspace frame.
  if (!ready || !document.documentElement.classList.contains('workspace-switch-pending')) return;
  const transition = JSON.parse(sessionStorage.getItem('workspace-transition') || 'null') as {
    name: string;
    action: 'switch' | 'create';
    startedAt: number;
  } | null;
  await nextTick();
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
  const remaining = Math.max(0, 2000 - (Date.now() - (transition?.startedAt ?? Date.now())));
  if (remaining) await new Promise((resolve) => setTimeout(resolve, remaining));
  document.documentElement.classList.add('workspace-switch-ready');
  setTimeout(
    () => {
      document.documentElement.classList.remove(
        'workspace-switch-pending',
        'workspace-switch-ready',
      );
      sessionStorage.removeItem('workspace-transition');
      if (
        transition &&
        !workspace.error &&
        !preferences.loadError &&
        !issues.loadError &&
        !projects.loadError &&
        !views.loadError
      )
        notifySuccess(
          transition.action === 'create' ? 'Workspace created' : 'Workspace switched',
          transition.name,
        );
    },
    matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : 240,
  );
});

watch(
  () => workspace.ready && workspace.configured,
  (configured) => {
    if (configured) {
      preferences.bootstrap();
      issues.bootstrap();
      labels.bootstrap();
      projects.bootstrap();
      views.bootstrap();
    }
  },
  { immediate: true },
);
</script>

<template>
  <MotionConfig reduced-motion="user">
    <div
      v-if="
        !workspace.ready ||
        workspace.error ||
        preferences.loadError ||
        issues.loadError ||
        labels.error ||
        projects.loadError ||
        views.loadError ||
        !workspace.configured ||
        !preferences.ready ||
        !issues.ready ||
        !labels.ready ||
        !projects.ready ||
        !views.ready
      "
      class="flex h-dvh min-h-0 flex-col overflow-hidden bg-workspace"
    >
      <AppTitlebar title="Welcome" :sidebar-visible="false" />
      <main
        v-if="
          !workspace.ready ||
          (workspace.configured &&
            (!preferences.ready ||
              !issues.ready ||
              !labels.ready ||
              !projects.ready ||
              !views.ready) &&
            !preferences.loadError &&
            !issues.loadError &&
            !labels.error &&
            !projects.loadError &&
            !views.loadError)
        "
        class="flex min-h-0 flex-1 items-center justify-center text-sm text-foreground-muted"
      >
        Opening workspace…
      </main>
      <main
        v-else-if="
          workspace.error ||
          preferences.loadError ||
          issues.loadError ||
          labels.error ||
          projects.loadError ||
          views.loadError
        "
        class="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 p-6 text-sm text-foreground"
      >
        <p role="alert">
          {{
            workspace.error ||
            preferences.loadError ||
            issues.loadError ||
            labels.error ||
            projects.loadError ||
            views.loadError
          }}
        </p>
        <button
          class="rounded-md bg-muted px-3 py-2"
          type="button"
          @click="
            workspace.error
              ? workspace.bootstrap()
              : preferences.loadError
                ? preferences.bootstrap()
                : issues.loadError
                  ? issues.bootstrap()
                  : labels.error
                    ? labels.bootstrap()
                    : projects.loadError
                      ? projects.bootstrap()
                      : views.bootstrap()
          "
        >
          Retry
        </button>
      </main>
      <WorkspaceSetup v-else />
    </div>
    <AppShell v-else />
    <Dialog
      :open="restoreReviewOpen"
      title="Review restored workspace"
      description="Cycle schedules have not advanced since this backup. Apply catch-up when ready."
      size="sm"
      :close-on-backdrop-click="false"
      @update:open="keepRestoreReviewOpen"
    >
      <template #header="{ titleId }">
        <div>
          <h2 :id="titleId" class="text-sm font-semibold text-foreground-highlighted">
            Review restored workspace
          </h2>
          <p class="mt-1 text-sm text-foreground-muted">
            Cycle schedules have not advanced since this backup. Apply catch-up when ready.
          </p>
        </div>
      </template>
      <template #body>
        <p v-if="catchupProgress" class="text-sm text-foreground-muted">{{ catchupProgress }}</p>
        <p v-if="catchupError" role="alert" class="text-sm text-danger">{{ catchupError }}</p>
      </template>
      <template #footer>
        <Button
          label="Apply cycle catch-up"
          size="sm"
          :disabled="catchupBusy"
          @click="completeRestore"
        />
      </template>
    </Dialog>
    <Dialog
      :open="Boolean(currentTimerNotice)"
      title="Review tracked time"
      size="sm"
      :close-on-backdrop-click="false"
      @update:open="() => {}"
    >
      <template #body>
        <p class="text-sm text-foreground">
          {{ currentTimerNotice?.issueIdentifier }}:
          <template v-if="currentTimerNotice?.kind === 'closed'">
            Time while the app was closed was included. You can review and edit this entry from the
            issue.
          </template>
          <template v-else-if="currentTimerNotice?.kind === 'long_gap'">
            The app was closed for over 24 hours. This time was paused for your review.
          </template>
          <template v-else-if="currentTimerNotice?.kind === 'backward_clock'">
            The system clock moved backward. No closed time was added; review this entry on the
            issue.
          </template>
          <template v-else>
            A sleep gap could not be measured reliably. The timer stopped at its last checkpoint.
          </template>
        </p>
        <p v-if="timerReviewError" role="alert" class="mt-3 text-sm text-danger">
          {{ timerReviewError }}
        </p>
      </template>
      <template #footer>
        <Button
          v-if="currentTimerNotice?.kind === 'long_gap'"
          label="Include elapsed time"
          size="sm"
          variant="ghost"
          :disabled="timerReviewBusy"
          @click="resolveTimerNotice(true)"
        />
        <Button label="Done" size="sm" :disabled="timerReviewBusy" @click="resolveTimerNotice()" />
      </template>
    </Dialog>
    <Dialog
      :open="closeRequested && Boolean(closeError)"
      title="Could not save before closing"
      size="sm"
      :close-on-backdrop-click="false"
      @update:open="() => {}"
    >
      <template #body>
        <p role="alert" class="text-sm text-foreground-muted">{{ closeError }}</p>
        <p class="mt-2 text-sm text-foreground-muted">
          Keep the app open to preserve your work, retry saving, or close with local recovery
          drafts.
        </p>
      </template>
      <template #footer>
        <Button label="Keep open" size="sm" variant="ghost" @click="closeRequested = false" />
        <Button label="Force close" size="sm" variant="ghost" @click="finishClose(true)" />
        <Button label="Retry save" size="sm" :disabled="closing" @click="finishClose()" />
      </template>
    </Dialog>
    <Toast :manager="notifications" position="bottom-right" />
    <AppUpdater />
  </MotionConfig>
</template>
