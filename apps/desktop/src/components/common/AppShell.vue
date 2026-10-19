<script setup lang="ts">
import { useMediaQuery } from '@vueuse/core';
import { computed, defineAsyncComponent, nextTick, ref, watch } from 'vue';
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router';
import { defineShortcuts } from '../../composables/defineShortcuts';
import { appShortcuts } from '../../lib/app-shortcuts';
import { copyIssueBranchName } from '../../lib/issue-branch';
import { dispatchIssueDetailAction } from '../../lib/issue-detail-command';
import type { IssueDetailAction } from '../../lib/issue-detail-command';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useUiStore } from '../../stores/ui';
import { useIssuesStore } from '../../stores/issues';
import { usePreferencesStore } from '../../stores/preferences';
import { useViewsStore } from '../../stores/views';
import { useWorkspaceStore } from '../../stores/workspace';
import AppSidebar from './AppSidebar.vue';
import AppTitlebar from './AppTitlebar.vue';
import CommandBar from './CommandBar.vue';
import ShortcutTooltip from './ShortcutTooltip.vue';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';

const ui = useUiStore();
const issues = useIssuesStore();
const preferences = usePreferencesStore();
const views = useViewsStore();
const workspace = useWorkspaceStore();
const route = useRoute();
const router = useRouter();

issues.ensureStatusGroups();

const CreateIssueModal = defineAsyncComponent(() => import('../issues/CreateIssueModal.vue'));

const issueModalLoaded = ref(false);
const mobileOpen = ref(false);
const sidebarCollapsed = ref(false);

const wideScreen = useMediaQuery('(min-width: 768px)');

watch(
  () => ui.createIssueOpen,
  (value) => {
    if (value) issueModalLoaded.value = true;
  },
  { immediate: true },
);

const sidebarVisible = computed(() =>
  wideScreen.value ? !sidebarCollapsed.value : mobileOpen.value,
);

const location = computed(() =>
  route.name === 'issue-full' || route.name === 'issue'
    ? String(route.params.id)
    : route.name === 'saved-view'
      ? (views.items.find((view) => view.id === route.params.viewId)?.name ?? 'Views')
      : route.name === 'views'
        ? 'Views'
        : route.name === 'projects'
          ? 'Projects'
          : route.name === 'settings'
            ? 'Settings'
            : route.name === 'triage'
              ? 'Triage'
              : route.name === 'archives'
                ? 'Archives'
                : route.name === 'cycles'
                  ? 'Cycles'
                  : route.name === 'time'
                    ? 'Time tracking'
                    : String(route.params.project || 'All issues'),
);

const breadcrumbParent = computed(() => {
  if (route.name === 'issue-full' || route.name === 'issue')
    return { label: 'All issues', to: '/issues' };
  if (route.name === 'saved-view') return { label: 'Views', to: '/views' };
  if (route.name === 'project-overview' || route.name === 'project-issues')
    return { label: 'Projects', to: '/projects' };
  return { label: workspace.workspaceName, to: '/issues' };
});

function toggleSidebar() {
  if (wideScreen.value) sidebarCollapsed.value = !sidebarCollapsed.value;
  else mobileOpen.value = !mobileOpen.value;
}

function openCommands(mode: typeof ui.commandBarMode = 'all') {
  ui.commandBarMode = mode;
  ui.commandBarOpen = true;
}

function openCreateIssue() {
  const project = typeof route.params.project === 'string' ? route.params.project : null;
  ui.createIssueDefaults = project ? { project } : null;
  ui.createIssueOpen = true;
}

async function openCreateView() {
  await router.push('/views');
  await nextTick();
  window.dispatchEvent(new Event('create-view-command'));
}

function issueCommand(name: string) {
  // The shell owns shortcuts; mounted collection views decide how to handle each action.
  window.dispatchEvent(new CustomEvent('issue-collection-command', { detail: name }));
}

function issueDetailAction(action: IssueDetailAction) {
  dispatchIssueDetailAction(String(route.params.id), action);
}

function projectCommand(name: string) {
  window.dispatchEvent(new CustomEvent('project-command', { detail: name }));
}

const inIssueView = computed(() =>
  ['issues', 'issue', 'triage', 'saved-view', 'project-issues'].includes(String(route.name)),
);

const inProjectOverview = computed(() => route.name === 'project-overview');

const inIssueDetail = computed(() => route.name === 'issue' || route.name === 'issue-full');

defineShortcuts(
  computed(() => ({
    [appShortcuts.commandBar]: { handler: () => openCommands(), usingInput: true },
    [appShortcuts.search]: () => openCommands(),
    [appShortcuts.createIssue]: openCreateIssue,
    [appShortcuts.createProject]: () => window.dispatchEvent(new Event('create-project-command')),
    [appShortcuts.createCycle]: async () => {
      await router.push('/cycles');
      window.dispatchEvent(new Event('create-cycle-command'));
    },
    [appShortcuts.createWorkspace]: () =>
      window.dispatchEvent(new CustomEvent('workspace-command', { detail: { action: 'create' } })),
    [appShortcuts.createView]: () => {
      void openCreateView();
    },
    [appShortcuts.switchWorkspace]: () => openCommands('workspaces'),
    [appShortcuts.allIssues]: () => router.push('/issues'),
    [appShortcuts.triage]: () => router.push('/triage'),
    [appShortcuts.archives]: () => router.push('/archives'),
    [appShortcuts.cycles]: () => router.push('/cycles'),
    [appShortcuts.time]: usesNativeStorage ? () => router.push('/time') : false,
    [appShortcuts.settings]: () => router.push('/settings'),
    [appShortcuts.workspaceLabels]: () => router.push('/settings#labels'),
    [appShortcuts.workflowSettings]: () => router.push('/settings#workflow'),
    [appShortcuts.issueBoard]: () => {
      void router.push('/issues').then(() => issueCommand('board'));
    },
    [appShortcuts.views]: () => router.push('/views'),
    [appShortcuts.openIssue]: () => openCommands('issues'),
    [appShortcuts.openProject]: () => openCommands('projects'),
    [appShortcuts.shortcuts]: () => openCommands('shortcuts'),
    [appShortcuts.filter]: inIssueView.value ? () => issueCommand('filter') : false,
    [appShortcuts.clearFilters]: inIssueView.value ? () => issueCommand('clearFilters') : false,
    [appShortcuts.findInView]: inIssueView.value ? () => issueCommand('find') : false,
    [appShortcuts.display]: inIssueView.value ? () => issueCommand('display') : false,
    [appShortcuts.toggleBoard]: inIssueView.value ? () => issueCommand('toggleBoard') : false,
    [appShortcuts.showBoard]: inIssueView.value ? () => issueCommand('board') : false,
    [appShortcuts.showList]: inIssueView.value ? () => issueCommand('list') : false,
    [appShortcuts.issueOverview]: inIssueView.value ? () => issueCommand('overview') : false,
    [appShortcuts.issueStatus]: inIssueDetail.value ? () => issueDetailAction('status') : false,
    [appShortcuts.issuePriority]: inIssueDetail.value ? () => issueDetailAction('priority') : false,
    [appShortcuts.issueEstimate]: inIssueDetail.value ? () => issueDetailAction('estimate') : false,
    [appShortcuts.issueRelation]: inIssueDetail.value ? () => issueDetailAction('relation') : false,
    [appShortcuts.issueParent]: inIssueDetail.value ? () => issueDetailAction('parent') : false,
    [appShortcuts.issueLabels]: inIssueDetail.value ? () => issueDetailAction('labels') : false,
    [appShortcuts.issueProject]: inIssueDetail.value ? () => issueDetailAction('project') : false,
    [appShortcuts.issueEdit]: inIssueDetail.value ? () => issueDetailAction('edit') : false,
    [appShortcuts.issueArchive]: inIssueDetail.value ? () => issueDetailAction('archive') : false,
    [appShortcuts.copyBranchName]: inIssueDetail.value
      ? () => {
          const issue = issues.items.find((item) => item.id === route.params.id);
          if (issue) void copyIssueBranchName(issue);
        }
      : false,
    [appShortcuts.createSubissue]: inIssueDetail.value
      ? () => issueDetailAction('subissue')
      : false,
    [appShortcuts.projectStatus]: inProjectOverview.value ? () => projectCommand('status') : false,
    [appShortcuts.projectIcon]: inProjectOverview.value
      ? () =>
          document
            .querySelector<HTMLElement>('[aria-label="Choose project icon and color"]')
            ?.click()
      : false,
    [appShortcuts.projectPriority]: inProjectOverview.value
      ? () => projectCommand('priority')
      : false,
    [appShortcuts.projectLabels]: inProjectOverview.value ? () => projectCommand('labels') : false,
    [appShortcuts.projectTargetDate]: inProjectOverview.value
      ? () => projectCommand('targetDate')
      : false,
    [appShortcuts.projectStartDate]: inProjectOverview.value
      ? () => projectCommand('startDate')
      : false,
    [appShortcuts.projectRename]: inProjectOverview.value ? () => projectCommand('rename') : false,
    [appShortcuts.projectResource]: inProjectOverview.value
      ? () => projectCommand('resource')
      : false,
    [appShortcuts.projectIssues]: inProjectOverview.value
      ? () => router.push({ name: 'project-issues', params: { project: route.params.project } })
      : false,
    [appShortcuts.projectLead]: inProjectOverview.value ? () => projectCommand('lead') : false,
    [appShortcuts.projectUpdate]: inProjectOverview.value ? () => projectCommand('update') : false,
    [appShortcuts.projectMilestone]: inProjectOverview.value
      ? () => projectCommand('milestone')
      : false,
  })),
  {
    enabled: (event) =>
      !ui.createIssueOpen &&
      !ui.commandBarOpen &&
      !document.querySelector('[role="dialog"]:not(.command-bar-dialog)') &&
      !(
        (event.metaKey || event.ctrlKey) &&
        event.key.toLowerCase() === 'k' &&
        event.target instanceof Element &&
        !!event.target.closest('[contenteditable="true"]')
      ),
  },
);
</script>

<template>
  <div
    class="workbench-shell flex h-dvh min-w-0 flex-col overflow-hidden bg-workspace font-sans text-foreground"
  >
    <AppTitlebar :title="location" :sidebar-visible="sidebarVisible" />
    <div class="relative flex min-h-0 flex-1">
      <AppSidebar
        v-model:open="mobileOpen"
        v-model:collapsed="sidebarCollapsed"
        :mobile="!wideScreen"
      />
      <div
        class="my-1.5 mr-1.5 flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-outline bg-default"
      >
        <div class="flex h-11 shrink-0 items-center gap-2 border-b border-outline px-3 text-xs">
          <ShortcutTooltip :label="sidebarVisible ? 'Hide sidebar' : 'Show sidebar'">
            <Button
              :icon="
                sidebarVisible
                  ? 'i-tabler-layout-sidebar-left-collapse'
                  : 'i-tabler-layout-sidebar-left-expand'
              "
              :aria-label="sidebarVisible ? 'Hide sidebar' : 'Show sidebar'"
              color="neutral"
              variant="ghost"
              @click="toggleSidebar"
            />
          </ShortcutTooltip>
          <Icon name="i-tabler-chevron-right" class="size-3 text-foreground-dimmed" />
          <RouterLink
            :to="breadcrumbParent.to"
            class="hidden min-w-0 truncate rounded px-1 py-1 text-foreground-muted hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info sm:inline"
            >{{ breadcrumbParent.label }}</RouterLink
          >
          <Icon
            name="i-tabler-chevron-right"
            class="hidden size-3 text-foreground-dimmed sm:inline"
          />
          <h1 class="truncate font-medium text-foreground-highlighted">{{ location }}</h1>
          <div v-if="route.name !== 'settings'" class="ml-auto">
            <ShortcutTooltip label="Create issue" shortcut="createIssue">
              <Button
                icon="i-tabler-plus"
                label="Create issue"
                aria-label="Create issue"
                color="neutral"
                variant="solid"
                size="xs"
                @click="openCreateIssue"
              />
            </ShortcutTooltip>
          </div>
        </div>
        <RouterView />
      </div>
    </div>
    <CreateIssueModal v-if="issueModalLoaded" v-model:open="ui.createIssueOpen" />
    <CommandBar v-model:open="ui.commandBarOpen" />
    <div
      v-if="issues.saveError"
      class="fixed right-3 bottom-3 z-50 flex max-w-sm items-center gap-3 rounded-lg border border-outline bg-elevated px-3 py-2 text-xs shadow-lg"
      role="alert"
    >
      <span>Issue change could not be saved. The last saved version was restored.</span>
      <Button
        icon="i-tabler-x"
        aria-label="Dismiss issue error"
        size="xs"
        variant="ghost"
        @click="issues.saveError = ''"
      />
    </div>
    <div
      v-if="preferences.saveError"
      class="fixed right-3 bottom-3 z-50 flex max-w-sm items-center gap-3 rounded-lg border border-outline bg-elevated px-3 py-2 text-xs shadow-lg"
      role="alert"
    >
      <span>Preferences could not be saved.</span>
      <Button label="Retry" size="xs" variant="outline" @click="preferences.retrySave()" />
    </div>
  </div>
</template>
