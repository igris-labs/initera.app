import { computed } from 'vue';
import type { Ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import type { CommentSearchResult, Issue } from '../../types/issue';
import type { ProjectRecord } from '../../lib/workspace-storage';
import type { PlanningNodeSearchResult } from '../../lib/issue-storage';
import type { Command } from '../../lib/command-search';
import { useIssuesStore } from '../../stores/issues';
import { useCyclesStore } from '../../stores/cycles';
import { useProjectsStore } from '../../stores/projects';
import { usePreferencesStore } from '../../stores/preferences';
import { useUiStore } from '../../stores/ui';
import { useViewsStore } from '../../stores/views';
import { useWorkspaceStore } from '../../stores/workspace';
import { useThemeChange } from '../../composables/useThemeChange';
import { baseCommands } from './command-base';
import { contextCommands } from './command-context';
import { commentCommand, recordCommands } from './command-records';

interface SearchRecords {
  nativeIssueRecords: Ref<Issue[]>;
  nativeProjectRecords: Ref<ProjectRecord[]>;
  nativePlanningMatches: Ref<PlanningNodeSearchResult[]>;
}

export function useCommandCatalog(open: Ref<boolean>, records: SearchRecords) {
  const router = useRouter();
  const route = useRoute();
  const ui = useUiStore();
  const issues = useIssuesStore();
  const cycles = useCyclesStore();
  const projects = useProjectsStore();
  const preferences = usePreferencesStore();
  const views = useViewsStore();
  const workspace = useWorkspaceStore();
  const { changeTheme } = useThemeChange();

  const projectName = computed(() =>
    route.name === 'project-overview' ? String(route.params.project) : '',
  );
  const issueName = computed(() =>
    route.name === 'issue' || route.name === 'issue-full' ? String(route.params.id) : '',
  );
  const currentIssue = computed(() => issues.items.find((item) => item.id === issueName.value));
  const mode = computed(() => ui.commandBarMode);

  const commands = computed<Command[]>(() => {
    const {
      navigation,
      actions: baseActions,
      viewActions,
      globalShortcuts,
      appearance,
      displaySettings,
    } = baseCommands({ router, route, ui, preferences, open, projectName, changeTheme });
    const actions = [
      ...contextCommands({ router, issues, projectName, issueName, currentIssue }),
      ...baseActions,
    ];
    const {
      issueResults,
      projectResults,
      planningResults,
      previewComments,
      viewResults,
      cycleResults,
      workspaceResults,
    } = recordCommands({ router, issues, projects, cycles, views, workspace, ...records });
    if (mode.value === 'issues') return [...issueResults, ...previewComments];
    if (mode.value === 'projects') return [...projectResults, ...planningResults];
    if (mode.value === 'workspaces')
      return [...actions.filter((item) => item.id === 'create-workspace'), ...workspaceResults];
    if (mode.value === 'shortcuts')
      return [...actions, ...viewActions, ...navigation, ...globalShortcuts].filter(
        (item) => item.shortcut,
      );
    return [
      ...actions,
      ...viewActions,
      ...navigation,
      ...appearance,
      ...displaySettings,
      ...workspaceResults,
      ...issueResults,
      ...projectResults,
      ...planningResults,
      ...previewComments,
      ...viewResults,
      ...cycleResults,
    ];
  });

  return {
    commands,
    mode,
    projectName,
    issueName,
    commentCommand: (comment: CommentSearchResult) => commentCommand(router, comment),
  };
}
