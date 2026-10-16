import { defineStore } from 'pinia';
import { loadPreferences, savePreferences, usesNativeStorage } from '../lib/workspace-storage';
import type { WorkspacePreferences } from '../lib/workspace-storage';

let saveQueue = Promise.resolve();
let lastQueued = '';
let pendingKeepTimerIssue: string | undefined;

export const usePreferencesStore = defineStore('preferences', {
  state: (): WorkspacePreferences & {
    ready: boolean;
    loadError: string;
    saveError: string;
    version: number;
  } => ({
    theme: 'dark',
    issueScope: 'active',
    issueLayout: 'list',
    boardGroupBy: 'status',
    groupIssuesByStatus: true,
    showIssuePriority: true,
    showIssueProject: true,
    estimateScale: 'fibonacci',
    timerOnExit: 'pause',
    concurrentTimers: false,
    timerOnSleep: 'pause',
    ready: !usesNativeStorage,
    loadError: '',
    saveError: '',
    version: 0,
  }),
  actions: {
    async flushSaves() {
      await saveQueue;
    },
    async bootstrap() {
      if (!usesNativeStorage || this.ready) return;
      this.loadError = '';
      try {
        const snapshot = await loadPreferences();
        this.$patch({ ...snapshot.preferences, version: snapshot.version });
        this.ready = true;
        this.$subscribe(() => this.persistCurrent(), { detached: true, flush: 'sync' });
      } catch (error) {
        this.loadError = String(error);
      }
    },
    persistCurrent() {
      if (!usesNativeStorage || !this.ready) return;
      const preferences: WorkspacePreferences = {
        theme: this.theme,
        issueScope: this.issueScope,
        issueLayout: this.issueLayout,
        boardGroupBy: this.boardGroupBy,
        groupIssuesByStatus: this.groupIssuesByStatus,
        showIssuePriority: this.showIssuePriority,
        showIssueProject: this.showIssueProject,
        estimateScale: this.estimateScale,
        timerOnExit: this.timerOnExit,
        concurrentTimers: this.concurrentTimers,
        timerOnSleep: this.timerOnSleep,
      };
      const key = JSON.stringify(preferences);
      if (key === lastQueued) return;
      lastQueued = key;
      const keepTimerIssue = pendingKeepTimerIssue;
      pendingKeepTimerIssue = undefined;
      saveQueue = saveQueue.then(async () => {
        try {
          const snapshot = await savePreferences(preferences, this.version, keepTimerIssue);
          this.version = snapshot.version;
          this.saveError = '';
        } catch (error) {
          this.saveError = String(error);
        }
      });
    },
    retrySave() {
      lastQueued = '';
      this.persistCurrent();
    },
    disableConcurrentTimers(keepTimerIssue: string) {
      pendingKeepTimerIssue = keepTimerIssue;
      this.concurrentTimers = false;
    },
  },
  persist: usesNativeStorage
    ? false
    : {
        pick: [
          'theme',
          'issueScope',
          'issueLayout',
          'boardGroupBy',
          'groupIssuesByStatus',
          'showIssuePriority',
          'showIssueProject',
          'estimateScale',
          'timerOnExit',
          'concurrentTimers',
          'timerOnSleep',
        ],
      },
});
