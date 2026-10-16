import { defineStore } from 'pinia';
import { setMutationWorkspace } from '../lib/native-invoke';
import {
  createWorkspace,
  loadWorkspace,
  saveWorkspaceIdentity,
  switchWorkspace,
  usesNativeStorage,
} from '../lib/workspace-storage';

const scopedKeys = [
  'issues',
  'projects',
  'preferences',
  'views',
  'project-updates',
  'cycles',
  'planning',
  'labels',
];

function movePreviewData(from: string, to: string) {
  for (const key of scopedKeys) {
    const value = localStorage.getItem(key);
    if (from && value !== null) localStorage.setItem(`workspace-data:${from}:${key}`, value);
    const next = localStorage.getItem(`workspace-data:${to}:${key}`);
    if (next === null) localStorage.removeItem(key);
    else localStorage.setItem(key, next);
  }
}

export const useWorkspaceStore = defineStore('workspace', {
  state: () => ({
    userName: '',
    workspaceName: '',
    workspaceId: '',
    workspaces: [] as { id: string; name: string }[],
    actorId: '',
    identityVersion: null as number | null,
    dataRoot: '',
    restoreError: '',
    restoredBackup: false,
    recoveredTimers: 0,
    ready: false,
    error: '',
  }),
  getters: {
    configured: (state) => Boolean(state.userName.trim() && state.workspaceName.trim()),
    initials: (state) =>
      state.userName
        .trim()
        .split(/\s+/)
        .slice(0, 2)
        .map((part) => part[0]?.toUpperCase() ?? '')
        .join(''),
  },
  actions: {
    async bootstrap() {
      if (!usesNativeStorage) {
        if (this.configured && !this.workspaceId) {
          this.workspaceId = crypto.randomUUID();
          this.workspaces = [{ id: this.workspaceId, name: this.workspaceName }];
        }

        if (this.configured && !this.actorId) this.actorId = 'preview-local-actor';

        this.ready = true;
        return;
      }

      this.ready = false;
      this.error = '';
      try {
        const result = await loadWorkspace();
        this.dataRoot = result.dataRoot;
        this.restoreError = result.restoreError ?? '';
        this.restoredBackup = result.restoredBackup;
        this.recoveredTimers = result.recoveredTimers;
        this.userName = result.identity?.userName ?? '';
        this.workspaceName = result.identity?.workspaceName ?? '';
        this.workspaceId = result.identity?.workspaceId ?? '';
        setMutationWorkspace(this.workspaceId);
        this.workspaces = result.workspaces;
        this.actorId = result.identity?.actorId ?? '';
        this.identityVersion = result.identityVersion;
      } catch (error) {
        this.error = String(error);
      } finally {
        this.ready = true;
      }
    },
    async saveIdentity(userName: string, workspaceName: string) {
      this.error = '';
      if (usesNativeStorage) {
        try {
          const { identity, version } = await saveWorkspaceIdentity(
            userName,
            workspaceName,
            this.identityVersion,
          );
          this.identityVersion = version;
          this.workspaceId = identity.workspaceId;
          setMutationWorkspace(this.workspaceId);
          this.actorId = identity.actorId;
          this.setIdentity(identity.userName, identity.workspaceName);
          if (!this.workspaces.some((item) => item.id === identity.workspaceId))
            this.workspaces.push({ id: identity.workspaceId, name: identity.workspaceName });
        } catch (error) {
          this.error = String(error);
          throw error;
        }

        return;
      }

      this.setIdentity(userName, workspaceName);
      this.workspaceId = crypto.randomUUID();
      this.workspaces = [{ id: this.workspaceId, name: this.workspaceName }];
    },
    async createWorkspace(name: string) {
      const trimmed = name.trim();
      if (!trimmed || trimmed.length > 80)
        throw new Error('Enter a workspace name of up to 80 characters.');
      if (this.workspaces.some((item) => item.name.toLowerCase() === trimmed.toLowerCase()))
        throw new Error('A workspace with this name already exists.');
      if (usesNativeStorage) {
        await createWorkspace(trimmed);
      } else {
        const id = crypto.randomUUID();
        localStorage.setItem(`workspace-data:${id}:issues`, JSON.stringify({ items: [] }));
        localStorage.setItem(`workspace-data:${id}:projects`, JSON.stringify({ items: [] }));
        localStorage.setItem(
          `workspace-data:${id}:project-updates`,
          JSON.stringify({ byProject: {} }),
        );
        movePreviewData(this.workspaceId, id);
        this.workspaces.push({ id, name: trimmed });
        this.workspaceId = id;
        this.workspaceName = trimmed;
      }

      location.hash = '#/issues';
      location.reload();
    },
    async switchWorkspace(id: string) {
      if (id === this.workspaceId) return;
      if (!this.workspaces.some((item) => item.id === id)) throw new Error('Workspace not found.');
      if (usesNativeStorage) await switchWorkspace(id);
      else {
        movePreviewData(this.workspaceId, id);
        this.workspaceId = id;
        this.workspaceName = this.workspaces.find((item) => item.id === id)!.name;
      }

      location.hash = '#/issues';
      location.reload();
    },
    setIdentity(userName: string, workspaceName: string) {
      this.userName = userName.trim();
      this.workspaceName = workspaceName.trim();
      const current = this.workspaces.find((item) => item.id === this.workspaceId);
      if (current) current.name = this.workspaceName;
    },
  },
  persist: usesNativeStorage
    ? false
    : { pick: ['userName', 'workspaceName', 'workspaceId', 'workspaces'] },
});
