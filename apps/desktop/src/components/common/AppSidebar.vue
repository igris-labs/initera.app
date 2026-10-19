<script setup lang="ts">
import { useEventListener } from '@vueuse/core';
import { computed, defineAsyncComponent, ref, watch } from 'vue';
import { RouterLink, useRoute, useRouter } from 'vue-router';
import type { MenuItem } from 'akaza-ui';
import { useIssuesStore } from '../../stores/issues';
import { useThemeChange } from '../../composables/useThemeChange';
import { useUiStore } from '../../stores/ui';
import { useViewsStore } from '../../stores/views';
import { useWorkspaceStore } from '../../stores/workspace';
import { useProjectsStore } from '../../stores/projects';
import { usePreferencesStore } from '../../stores/preferences';
import { defaultProjectIcon, iconColor } from '../../lib/icon-presentation';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { notifications } from '../../lib/notifications';
import { checkForUpdates } from '../../lib/app-updater';
import { shortcutHint } from '../../lib/app-shortcuts';
import ShortcutTooltip from './ShortcutTooltip.vue';
import RunningTimer from './RunningTimer.vue';
import AppAboutDialog from './AppAboutDialog.vue';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { DropdownMenu } from '../ui/dropdown-menu';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Sidebar } from '../ui/sidebar';

interface Props {
  mobile: boolean;
}

defineProps<Props>();

const CreateProjectDialog = defineAsyncComponent(
  () => import('../projects/CreateProjectDialog.vue'),
);

const open = defineModel<boolean>('open', { required: true });
const collapsed = defineModel<boolean>('collapsed', { required: true });

const route = useRoute();
const router = useRouter();
const issues = useIssuesStore();
const ui = useUiStore();
const views = useViewsStore();
const workspace = useWorkspaceStore();
const projectStore = useProjectsStore();
const preferences = usePreferencesStore();

const projectDialogOpen = ref(false);
const projectDialogLoaded = ref(false);
const workspaceDialogOpen = ref(false);
const newWorkspaceName = ref('');
const workspaceError = ref('');
const aboutOpen = ref(false);

const workspaceItems = computed<MenuItem[]>(() =>
  workspace.workspaces.map((item) => ({
    label: item.name,
    value: item.id,
    slot: 'workspace',
    onSelect: () => selectWorkspace(item.id),
  })),
);

function showWorkspaceTransition(name: string, action: 'switch' | 'create') {
  const shell = document.querySelector('.workbench-shell') ?? document.body;
  const styles = getComputedStyle(shell);
  const transition = {
    name,
    action,
    startedAt: Date.now(),
    background: styles.backgroundColor,
    foreground: styles.color,
  };
  sessionStorage.setItem('workspace-transition', JSON.stringify(transition));
  document.documentElement.style.setProperty('--workspace-switch-color', transition.background);
  document.documentElement.style.setProperty(
    '--workspace-switch-foreground',
    transition.foreground,
  );
  document.getElementById('workspace-transition-action')!.textContent =
    action === 'switch' ? 'Switching workspace to' : 'Creating workspace';
  document.getElementById('workspace-transition-target')!.textContent = name;
  document.documentElement.classList.add('workspace-switch-pending');
}

function uncoverFailedSwitch() {
  sessionStorage.removeItem('workspace-transition');
  document.documentElement.classList.remove('workspace-switch-pending');
}

async function selectWorkspace(id: string) {
  if (id === workspace.workspaceId) return;
  const name = workspace.workspaces.find((item) => item.id === id)?.name;
  if (!name) return;
  showWorkspaceTransition(name, 'switch');
  try {
    await Promise.all([issues.flushSaves(), preferences.flushSaves(), views.flushSaves()]);
    await workspace.switchWorkspace(id);
  } catch (error) {
    uncoverFailedSwitch();
    workspaceError.value = error instanceof Error ? error.message : String(error);
    notifications.add({
      title: 'Could not switch workspace',
      description: workspaceError.value,
      type: 'error',
    });
  }
}

async function addWorkspace() {
  workspaceError.value = '';
  const name = newWorkspaceName.value.trim();
  if (name) showWorkspaceTransition(name, 'create');
  try {
    await Promise.all([issues.flushSaves(), preferences.flushSaves(), views.flushSaves()]);
    await workspace.createWorkspace(newWorkspaceName.value);
  } catch (error) {
    uncoverFailedSwitch();
    workspaceError.value = error instanceof Error ? error.message : String(error);
  }
}

useEventListener(window, 'workspace-command', (event) => {
  const detail = (event as CustomEvent<{ action: 'create' | 'switch'; id?: string }>).detail;
  if (detail.action === 'create') workspaceDialogOpen.value = true;
  else if (detail.id) void selectWorkspace(detail.id);
});

useEventListener(window, 'create-project-command', () => {
  projectDialogOpen.value = true;
});

watch(projectDialogOpen, (value) => {
  if (value) projectDialogLoaded.value = true;
});

const { theme, changeTheme } = useThemeChange();

const accountItems = computed<MenuItem[]>(() => [
  {
    label: 'Command bar',
    value: 'command-bar',
    slot: 'command-bar',
    onSelect: () => (ui.commandBarOpen = true),
  },
  {
    label: 'Settings',
    value: 'settings',
    slot: 'settings',
    onSelect: () => router.push('/settings'),
  },
  {
    label: 'Theme',
    value: 'theme',
    slot: 'theme',
    children: (['system', 'light', 'dark'] as const).map((value) => ({
      label: `${value[0].toUpperCase()}${value.slice(1)}`,
      value: `theme-${value}`,
      slot: `theme-${value}`,
      onSelect: () =>
        changeTheme(value, document.querySelector('[aria-label="Help and settings"]')),
    })),
  },
  {
    label: 'Help',
    value: 'help',
    slot: 'help',
    children: [
      {
        label: 'About',
        value: 'about',
        slot: 'about',
        onSelect: () => (aboutOpen.value = true),
      },
      {
        label: 'Check for updates',
        value: 'check-for-updates',
        slot: 'check-for-updates',
        onSelect: () => void checkForUpdates(true),
      },
      {
        label: 'Submit Issue & Feedback',
        value: 'submit-feedback',
        slot: 'submit-feedback',
        onSelect: () => void openFeedback(),
      },
    ],
  },
]);

async function openFeedback() {
  const url = 'https://github.com/igris-labs/initera.app/issues/new';
  if (!usesNativeStorage) {
    window.open(url, '_blank', 'noopener,noreferrer');
    return;
  }

  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
  } catch (error) {
    notifications.add({
      title: 'Could not open GitHub Issues',
      description: String(error),
      type: 'error',
    });
  }
}

const projectCounts = computed(() =>
  Object.fromEntries(
    projectStore.items.map((project) => [
      project.name,
      issues.counts?.projects[project.name]?.total ??
        issues.items.filter((issue) => issue.project === project.name && !issue.triage).length,
    ]),
  ),
);

async function loadMoreSidebarProjects() {
  try {
    await projectStore.loadMore();
  } catch (error) {
    notifications.add({
      title: 'Could not load projects',
      description: String(error),
      type: 'error',
    });
  }
}
</script>

<template>
  <Sidebar
    v-model:open="open"
    v-model:collapsed="collapsed"
    :mobile="mobile"
    aria-label="Workspace navigation"
    class="z-20"
  >
    <template #header>
      <DropdownMenu
        :items="workspaceItems"
        class="w-[calc(var(--akaza-sidebar-size)-0.75rem)] max-w-[calc(100dvw-1rem)]"
      >
        <template #default="{ toggle, triggerProps }">
          <button
            v-bind="triggerProps"
            type="button"
            aria-label="Switch workspace"
            class="inline-flex min-h-7 max-w-full min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-0.5 text-left text-xs font-medium text-foreground transition-colors hover:bg-accented focus-visible:outline-2 focus-visible:outline-info aria-expanded:bg-accented"
            @click="toggle"
          >
            <span
              class="flex size-6 shrink-0 items-center justify-center rounded-md bg-inverted text-xs font-semibold text-foreground-inverted"
              >{{ workspace.workspaceName.charAt(0).toUpperCase() }}</span
            >
            <span class="min-w-0 truncate">{{ workspace.workspaceName }}</span>
            <Icon name="i-tabler-chevron-down" class="size-3.5 shrink-0 text-foreground-muted" />
          </button>
        </template>
        <template #workspace="{ item }">
          <span class="flex w-full min-w-0 items-center gap-2">
            <span class="min-w-0 flex-1 truncate">{{ item.label }}</span>
            <Icon
              v-if="item.value === workspace.workspaceId"
              name="i-tabler-check"
              class="size-3.5 shrink-0 text-info"
            />
          </span>
        </template>
      </DropdownMenu>
      <ShortcutTooltip label="Add workspace" shortcut="createWorkspace">
        <Button
          icon="i-tabler-plus"
          aria-label="Add workspace"
          color="neutral"
          variant="ghost"
          @click="workspaceDialogOpen = true"
        />
      </ShortcutTooltip>
      <Button
        icon="i-tabler-x"
        aria-label="Hide sidebar"
        color="neutral"
        variant="ghost"
        class="md:hidden"
        @click="open = false"
      />
    </template>

    <nav class="space-y-0.5 px-2 pt-1" aria-label="Primary">
      <RouterLink
        to="/issues"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.path === '/issues' || route.name === 'issue' || route.name === 'issue-full'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-list" class="size-4" /><span class="flex-1">All issues</span
        ><span class="text-xs tabular-nums text-foreground-dimmed">{{ issues.activeCount }}</span>
      </RouterLink>
      <RouterLink
        to="/triage"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.path === '/triage'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-inbox" class="size-4" /><span class="flex-1">Triage</span
        ><span class="text-xs tabular-nums text-foreground-dimmed">{{ issues.triageCount }}</span>
      </RouterLink>
      <RouterLink
        to="/archives"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.name === 'archives'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-archive" class="size-4" /><span class="flex-1">Archives</span>
        <span class="text-xs tabular-nums text-foreground-dimmed">{{
          issues.counts?.archived ?? issues.archivedItems.length
        }}</span>
      </RouterLink>
    </nav>

    <div class="px-4 pt-4 pb-1 text-xs text-foreground-dimmed">Workspace</div>
    <nav class="space-y-0.5 px-2" aria-label="Workspace views">
      <RouterLink
        to="/cycles"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.name === 'cycles'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-calendar-time" class="size-4" /><span class="flex-1">Cycles</span>
      </RouterLink>
      <RouterLink
        v-if="usesNativeStorage"
        to="/time"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.name === 'time'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-clock" class="size-4" /><span class="flex-1">Time tracking</span>
      </RouterLink>
      <RouterLink
        to="/projects"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.name === 'projects'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-cube" class="size-4" /><span class="flex-1">Projects</span>
      </RouterLink>
      <RouterLink
        to="/views"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.name === 'views' || route.name === 'saved-view'
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon name="i-tabler-stack-2" class="size-4" /><span class="flex-1">Views</span>
      </RouterLink>
      <RouterLink
        v-for="view in views.items.filter((item) => item.favorite)"
        :key="view.id"
        :to="{ name: 'saved-view', params: { viewId: view.id } }"
        class="flex min-h-7 items-center gap-2 rounded-md px-2 pl-7 text-xs text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted"
      >
        <Icon name="i-tabler-star-filled" class="size-3 text-info" /><span
          class="min-w-0 flex-1 truncate"
          >{{ view.name }}</span
        >
      </RouterLink>
    </nav>

    <div class="flex items-center justify-between px-4 pt-4 pb-1 text-xs text-foreground-dimmed">
      <span>Projects</span>
      <ShortcutTooltip label="Create project" shortcut="createProject">
        <Button
          icon="i-tabler-plus"
          variant="ghost"
          size="xs"
          aria-label="Create project"
          @click="projectDialogOpen = true"
        />
      </ShortcutTooltip>
    </div>
    <nav class="space-y-0.5 px-2" aria-label="Projects">
      <RouterLink
        v-for="project in projectStore.items"
        :key="project.id"
        :to="{ name: 'project-overview', params: { project: project.name } }"
        :class="[
          'flex min-h-(--ui-sidebar-row-height) items-center gap-2 rounded-md px-2 py-1 text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out',
          route.params.project === project.name
            ? 'bg-(--ui-nav-active) text-foreground-highlighted'
            : 'text-foreground-muted hover:bg-accented/70 hover:text-foreground-highlighted',
        ]"
      >
        <Icon
          :name="project.icon || defaultProjectIcon"
          :class="['size-4', iconColor(project.color).text]"
        /><span class="flex-1">{{ project.name }}</span
        ><span class="text-xs tabular-nums text-foreground-dimmed">{{
          projectCounts[project.name] ?? 0
        }}</span>
      </RouterLink>
      <Button
        v-if="projectStore.nextCursor"
        label="Load more projects"
        size="xs"
        variant="ghost"
        class="w-full justify-start"
        @click="loadMoreSidebarProjects"
      />
    </nav>

    <CreateProjectDialog v-if="projectDialogLoaded" v-model:open="projectDialogOpen" />
    <Dialog v-model:open="workspaceDialogOpen" title="Add workspace" size="sm">
      <template #body>
        <form id="add-workspace-form" class="space-y-3" @submit.prevent="addWorkspace">
          <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
            Workspace name
            <Input
              v-model="newWorkspaceName"
              aria-label="Workspace name"
              autofocus
              placeholder="e.g. Design team"
              maxlength="80"
            />
          </label>
          <p v-if="workspaceError" role="alert" class="text-xs text-danger">
            {{ workspaceError }}
          </p>
        </form>
      </template>
      <template #footer>
        <Button label="Cancel" variant="ghost" size="sm" @click="workspaceDialogOpen = false" />
        <Button label="Create workspace" size="sm" type="submit" form="add-workspace-form" />
      </template>
    </Dialog>

    <template #footer>
      <RunningTimer v-if="usesNativeStorage" />
      <DropdownMenu
        :items="accountItems"
        side="top"
        align="start"
        :teleport="false"
        size="sm"
        class="w-max min-w-full max-w-[calc(100dvw-1rem)]"
      >
        <template #default="{ toggle, triggerProps }">
          <button
            v-bind="triggerProps"
            type="button"
            aria-label="Help and settings"
            title="Help and settings"
            class="group flex min-h-8 w-full cursor-pointer items-center gap-2 rounded-md border border-outline/60 bg-muted/40 px-2 text-left text-xs text-foreground transition-[background-color,border-color,color] duration-150 ease-out hover:border-outline hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info aria-expanded:border-outline aria-expanded:bg-accented aria-expanded:text-foreground-highlighted"
            @click="toggle"
          >
            <span
              class="flex size-5 shrink-0 items-center justify-center rounded-full bg-accented text-[10px] font-semibold text-foreground-highlighted"
              >{{ workspace.initials }}</span
            >
            <span class="min-w-0 flex-1 truncate">{{ workspace.userName }}</span>
            <Icon
              name="i-tabler-chevron-up"
              class="size-3.5 shrink-0 transition-transform duration-150 group-aria-expanded:rotate-180"
            />
          </button>
        </template>
        <template #command-bar>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-command" class="size-4 text-foreground-muted" />
            <span class="flex-1 whitespace-nowrap">Command bar</span>
            <kbd
              class="rounded-md border border-outline bg-muted px-1.5 py-0.5 text-xs font-medium whitespace-nowrap text-foreground-highlighted"
              >{{ shortcutHint('commandBar') }}</kbd
            >
          </span>
        </template>
        <template #settings>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-settings" class="size-4 text-foreground-muted" />
            <span class="flex-1">Settings</span>
          </span>
        </template>
        <template #theme>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-palette" class="size-4 text-foreground-muted" />
            <span class="flex-1">Theme</span>
            <Icon name="i-tabler-chevron-right" class="size-3.5 text-foreground-muted" />
          </span>
        </template>
        <template #help>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-bulb" class="size-4 text-foreground-muted" />
            <span class="flex-1">Help</span>
            <Icon name="i-tabler-chevron-right" class="size-3.5 text-foreground-muted" />
          </span>
        </template>
        <template #about>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-info-circle" class="size-4 text-foreground-muted" />
            <span class="flex-1">About</span>
          </span>
        </template>
        <template #check-for-updates>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-refresh" class="size-4 text-foreground-muted" />
            <span class="flex-1">Check for updates</span>
          </span>
        </template>
        <template #submit-feedback>
          <span class="flex w-full items-center gap-2">
            <Icon name="i-tabler-bug" class="size-4 text-foreground-muted" />
            <span class="flex-1">Submit Issue &amp; Feedback</span>
          </span>
        </template>
        <template
          v-for="value in ['system', 'light', 'dark'] as const"
          :key="value"
          #[`theme-${value}`]
        >
          <span class="flex w-full items-center gap-2">
            <Icon
              :name="
                value === 'system'
                  ? 'i-tabler-device-desktop'
                  : value === 'light'
                    ? 'i-tabler-sun'
                    : 'i-tabler-moon'
              "
              class="size-4 text-foreground-muted"
            />
            <span class="flex-1">{{ value[0].toUpperCase() + value.slice(1) }}</span>
            <Icon v-if="theme === value" name="i-tabler-check" class="size-4 text-info" />
          </span>
        </template>
      </DropdownMenu>
      <AppAboutDialog v-model:open="aboutOpen" />
    </template>
  </Sidebar>
</template>
