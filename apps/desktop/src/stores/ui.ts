import { defineStore } from 'pinia';
import type { IssueDraft } from '../types/issue';

export const useUiStore = defineStore('ui', {
  state: (): {
    createIssueOpen: boolean;
    createIssueDefaults: Partial<
      Pick<IssueDraft, 'status' | 'priority' | 'project' | 'parentId' | 'milestoneId'>
    > | null;
    commandBarOpen: boolean;
    commandBarMode: 'all' | 'issues' | 'projects' | 'workspaces' | 'shortcuts';
  } => ({
    createIssueOpen: false,
    createIssueDefaults: null,
    commandBarOpen: false,
    commandBarMode: 'all',
  }),
});
