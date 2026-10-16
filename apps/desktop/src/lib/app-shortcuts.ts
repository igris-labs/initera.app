// Keep these keys in sync with AppShell's defineShortcuts bindings and command bar hints.
export const appShortcuts = {
  commandBar: 'meta_k',
  search: '/',
  createIssue: 'c',
  createProject: 'n-p',
  createCycle: 'n-c',
  createWorkspace: 'n-w',
  createView: 'n-v',
  switchWorkspace: 'o-w',
  allIssues: 'g-e',
  triage: 'g-t',
  archives: 'g-x',
  cycles: 'g-c',
  time: 'g-l',
  settings: 'g-s',
  workspaceLabels: 'g-a',
  workflowSettings: 'g-w',
  issueBoard: 'g-b',
  views: 'o-v',
  openIssue: 'o-i',
  openProject: 'o-p',
  filter: 'f',
  clearFilters: 'shift_f',
  findInView: 'meta_f',
  display: 'shift_v',
  toggleBoard: 'meta_b',
  showBoard: 'v-b',
  showList: 'v-l',
  issueOverview: 'v-o',
  issueStatus: 's',
  issuePriority: 'p',
  issueEstimate: 'shift_e',
  issueRelation: 'shift_r',
  issueParent: 'shift_h',
  issueLabels: 'l',
  issueProject: 'shift_p',
  issueEdit: 'e',
  issueArchive: 'shift_a',
  copyBranchName: 'meta_shift_b',
  createSubissue: 'shift_c',
  projectStatus: 'p-s',
  projectIcon: 'p-i',
  projectPriority: 'p-p',
  projectTargetDate: 'p-d',
  projectStartDate: 'p-a',
  projectRename: 'p-r',
  projectResource: 'p-f',
  projectIssues: 'p-v',
  projectLead: 'p-o',
  projectLabels: 'p-l',
  projectUpdate: 'p-u',
  projectMilestone: 'p-m',
  shortcuts: '?',
} as const;

export type AppShortcut = keyof typeof appShortcuts;

export function shortcutHint(shortcut: AppShortcut): string {
  const mac = typeof navigator !== 'undefined' && /Macintosh;/.test(navigator.userAgent);
  return appShortcuts[shortcut]
    .split('-')
    .map((part) =>
      part
        .split('_')
        .map((key) => {
          if (key === 'meta') return mac ? '⌘' : 'Ctrl';
          if (key === 'shift') return mac ? '⇧' : 'Shift';
          if (key === 'alt') return mac ? '⌥' : 'Alt';
          return key.toUpperCase();
        })
        .join(' + '),
    )
    .join(' then ');
}
