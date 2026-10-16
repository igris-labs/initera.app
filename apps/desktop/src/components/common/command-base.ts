import { nextTick } from 'vue';
import type { Ref } from 'vue';
import type { RouteLocationNormalizedLoadedGeneric, Router } from 'vue-router';
import type { Command } from '../../lib/command-search';
import { usesNativeStorage } from '../../lib/workspace-storage';
import type { useUiStore } from '../../stores/ui';
import type { usePreferencesStore } from '../../stores/preferences';

interface BaseCommandContext {
  router: Router;
  route: RouteLocationNormalizedLoadedGeneric;
  ui: ReturnType<typeof useUiStore>;
  preferences: ReturnType<typeof usePreferencesStore>;
  open: Ref<boolean>;
  projectName: Ref<string>;
  changeTheme: (value: 'system' | 'light' | 'dark') => void;
}

function sidebarAction(name: string, detail?: unknown) {
  window.dispatchEvent(new CustomEvent(name, { detail }));
}

function issueAction(name: string) {
  window.dispatchEvent(new CustomEvent('issue-collection-command', { detail: name }));
}

export function baseCommands({
  router,
  route,
  ui,
  preferences,
  open,
  projectName,
  changeTheme,
}: BaseCommandContext) {
  const navigation: Command[] = [
    {
      id: 'all-issues',
      label: 'All issues',
      section: 'Go to',
      icon: 'i-tabler-list',
      shortcut: 'allIssues',
      run: () => router.push('/issues'),
    },
    {
      id: 'triage',
      label: 'Triage',
      section: 'Go to',
      icon: 'i-tabler-inbox',
      shortcut: 'triage',
      run: () => router.push('/triage'),
    },
    {
      id: 'archives',
      label: 'Archives',
      section: 'Go to',
      icon: 'i-tabler-archive',
      shortcut: 'archives',
      run: () => router.push('/archives'),
    },
    {
      id: 'cycles',
      label: 'Cycles',
      section: 'Go to',
      icon: 'i-tabler-calendar-time',
      shortcut: 'cycles',
      run: () => router.push('/cycles'),
    },
    ...(usesNativeStorage
      ? [
          {
            id: 'time',
            label: 'Time tracking',
            section: 'Go to',
            icon: 'i-tabler-clock',
            shortcut: 'time' as const,
            run: () => router.push('/time'),
          },
        ]
      : []),
    {
      id: 'projects',
      label: 'Projects',
      section: 'Go to',
      icon: 'i-tabler-cube',
      shortcut: 'openProject',
      run: () => router.push('/projects'),
    },
    {
      id: 'views',
      label: 'Views',
      section: 'Go to',
      icon: 'i-tabler-stack-2',
      shortcut: 'views',
      run: () => router.push('/views'),
    },
    {
      id: 'settings',
      label: 'Settings',
      section: 'Go to',
      icon: 'i-tabler-adjustments-horizontal',
      shortcut: 'settings',
      run: () => router.push('/settings'),
    },
    {
      id: 'workspace-labels',
      label: 'Workspace labels',
      section: 'Go to',
      icon: 'i-tabler-tag',
      shortcut: 'workspaceLabels',
      run: () => router.push('/settings#labels'),
    },
    {
      id: 'workflow-settings',
      label: 'Workflow statuses',
      section: 'Go to',
      icon: 'i-tabler-circle-dashed',
      shortcut: 'workflowSettings',
      run: () => router.push('/settings#workflow'),
    },
  ];
  const actions: Command[] = [
    {
      id: 'create-issue',
      label: 'Create issue',
      section: 'Actions',
      icon: 'i-tabler-plus',
      shortcut: 'createIssue',
      run: () => {
        ui.createIssueDefaults = projectName.value ? { project: projectName.value } : null;
        ui.createIssueOpen = true;
      },
    },
    {
      id: 'create-project',
      label: 'Create project',
      section: 'Actions',
      icon: 'i-tabler-cube-plus',
      shortcut: 'createProject',
      run: () => sidebarAction('create-project-command'),
    },
    {
      id: 'create-cycle',
      label: 'Create cycle',
      section: 'Actions',
      icon: 'i-tabler-calendar-time',
      shortcut: 'createCycle',
      run: async () => {
        await router.push('/cycles');
        await nextTick();
        sidebarAction('create-cycle-command');
      },
    },
    {
      id: 'create-view',
      label: 'Create view',
      section: 'Actions',
      icon: 'i-tabler-stack-2',
      shortcut: 'createView',
      run: async () => {
        await router.push('/views');
        await nextTick();
        sidebarAction('create-view-command');
      },
    },
    {
      id: 'create-workspace',
      label: 'Create workspace',
      section: 'Actions',
      icon: 'i-tabler-plus',
      shortcut: 'createWorkspace',
      run: () => sidebarAction('workspace-command', { action: 'create' }),
    },
    {
      id: 'switch-workspace',
      label: 'Switch workspace',
      section: 'Actions',
      icon: 'i-tabler-switch-horizontal',
      shortcut: 'switchWorkspace',
      run: () => {
        ui.commandBarMode = 'workspaces';
        open.value = true;
      },
    },
    {
      id: 'keyboard-shortcuts',
      label: 'Keyboard shortcuts',
      section: 'Actions',
      icon: 'i-tabler-keyboard',
      shortcut: 'shortcuts',
      run: () => {
        ui.commandBarMode = 'shortcuts';
        open.value = true;
      },
    },
  ];
  const issueView = ['issues', 'issue', 'triage', 'saved-view', 'project-issues'].includes(
    String(route.name),
  );
  const viewActions: Command[] = issueView
    ? [
        {
          id: 'active-issues',
          label: 'Show active issues',
          section: 'This view',
          icon: 'i-tabler-circle-dot',
          run: () => issueAction('active'),
        },
        {
          id: 'all-issue-states',
          label: 'Show all issues including completed',
          section: 'This view',
          icon: 'i-tabler-list',
          run: () => issueAction('all'),
        },
        {
          id: 'show-board',
          label: 'Show board',
          section: 'This view',
          icon: 'i-tabler-layout-board',
          shortcut: 'showBoard',
          run: () => issueAction('board'),
        },
        {
          id: 'show-list',
          label: 'Show list',
          section: 'This view',
          icon: 'i-tabler-list-details',
          shortcut: 'showList',
          run: () => issueAction('list'),
        },
        {
          id: 'issue-overview',
          label: 'Toggle issue overview',
          section: 'This view',
          icon: 'i-tabler-layout-sidebar-right',
          shortcut: 'issueOverview',
          run: () => issueAction('overview'),
        },
        {
          id: 'filter',
          label: 'Filter issues',
          section: 'This view',
          icon: 'i-tabler-filter',
          shortcut: 'filter',
          run: () => issueAction('filter'),
        },
        {
          id: 'clear-filters',
          label: 'Clear filters',
          section: 'This view',
          icon: 'i-tabler-filter-off',
          shortcut: 'clearFilters',
          run: () => issueAction('clearFilters'),
        },
        {
          id: 'find',
          label: 'Find in view',
          section: 'This view',
          icon: 'i-tabler-search',
          shortcut: 'findInView',
          run: () => issueAction('find'),
        },
        {
          id: 'display',
          label: 'Display options',
          section: 'This view',
          icon: 'i-tabler-adjustments-horizontal',
          shortcut: 'display',
          run: () => issueAction('display'),
        },
        {
          id: 'toggle-board',
          label: 'Toggle board and list',
          section: 'This view',
          icon: 'i-tabler-layout-board',
          shortcut: 'toggleBoard',
          run: () => issueAction('toggleBoard'),
        },
      ]
    : [];
  const globalShortcuts: Command[] = [
    {
      id: 'open-command-bar',
      label: 'Open command bar',
      section: 'Global',
      icon: 'i-tabler-command',
      shortcut: 'commandBar',
      run: () => {
        open.value = true;
      },
    },
    {
      id: 'search-workspace',
      label: 'Search workspace',
      section: 'Global',
      icon: 'i-tabler-search',
      shortcut: 'search',
      run: () => {
        ui.commandBarMode = 'all';
        open.value = true;
      },
    },
    {
      id: 'issue-board',
      label: 'Go to issue board',
      section: 'Go to',
      icon: 'i-tabler-layout-board',
      shortcut: 'issueBoard',
      run: async () => {
        await router.push('/issues');
        issueAction('board');
      },
    },
    {
      id: 'open-issue',
      label: 'Find an issue',
      section: 'Global',
      icon: 'i-tabler-circle-dashed',
      shortcut: 'openIssue',
      run: () => {
        ui.commandBarMode = 'issues';
        open.value = true;
      },
    },
  ];
  const appearance: Command[] = (['system', 'light', 'dark'] as const).map((value) => ({
    id: `theme-${value}`,
    label: `Use ${value} theme`,
    section: 'Appearance',
    icon:
      value === 'system'
        ? 'i-tabler-device-desktop'
        : value === 'light'
          ? 'i-tabler-sun'
          : 'i-tabler-moon',
    run: () => changeTheme(value),
  }));
  const displaySettings: Command[] = [
    {
      id: 'group-by-status',
      label: `${preferences.groupIssuesByStatus ? 'Stop grouping' : 'Group'} issues by status`,
      section: 'Issue display',
      icon: 'i-tabler-category',
      keywords: 'settings preferences',
      run: () => {
        preferences.groupIssuesByStatus = !preferences.groupIssuesByStatus;
      },
    },
    {
      id: 'toggle-priority',
      label: `${preferences.showIssuePriority ? 'Hide' : 'Show'} issue priority`,
      section: 'Issue display',
      icon: 'i-tabler-antenna-bars-5',
      keywords: 'settings preferences',
      run: () => {
        preferences.showIssuePriority = !preferences.showIssuePriority;
      },
    },
    {
      id: 'toggle-project',
      label: `${preferences.showIssueProject ? 'Hide' : 'Show'} issue project`,
      section: 'Issue display',
      icon: 'i-tabler-cube',
      keywords: 'settings preferences',
      run: () => {
        preferences.showIssueProject = !preferences.showIssueProject;
      },
    },
  ];
  return { navigation, actions, viewActions, globalShortcuts, appearance, displaySettings };
}
