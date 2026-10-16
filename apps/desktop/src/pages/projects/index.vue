<script setup lang="ts">
import { watchDebounced } from '@vueuse/core';
import { computed, defineAsyncComponent, onMounted, ref, watch } from 'vue';
import { RouterLink } from 'vue-router';
import { useIssuesStore } from '../../stores/issues';
import { useProjectsStore } from '../../stores/projects';
import { useWorkspaceStore } from '../../stores/workspace';
import { Button } from '../../components/ui/button';
import { DataTable } from '../../components/ui/data-table';
import { Dialog } from '../../components/ui/dialog';
import { Icon } from '../../components/ui/icon';
import { Input } from '../../components/ui/input';
import { Select } from '../../components/ui/select';
import { defaultProjectIcon, iconColor } from '../../lib/icon-presentation';
import { previewProjectLifecycle, usesNativeStorage } from '../../lib/workspace-storage';
import type { ProjectIndexQuery, ProjectLifecyclePreview } from '../../lib/workspace-storage';

const projects = useProjectsStore();
const issues = useIssuesStore();
const workspace = useWorkspaceStore();

const createOpen = ref(false);
const showInactive = ref(false);
const lifecycleError = ref('');
const pagingError = ref('');
const pagingBusy = ref(false);
const sort = ref<ProjectIndexQuery['sort']>(projects.indexQuery.sort);
const statusFilter = ref(projects.indexQuery.status ?? 'All statuses');
const priorityFilter = ref(projects.indexQuery.priority ?? 'All priorities');
const search = ref(projects.indexQuery.search);
const lifecyclePreview = ref<ProjectLifecyclePreview | null>(null);
const lifecycleRequest = ref<{ id: string; name: string; action: 'archive' | 'trash' } | null>(
  null,
);

onMounted(
  () =>
    void projects.loadInactive().catch((error) => {
      lifecycleError.value = String(error);
    }),
);

async function requestLifecycle(id: string, name: string, action: 'archive' | 'trash') {
  lifecycleRequest.value = { id, name, action };
  lifecyclePreview.value = null;
  lifecycleError.value = '';
  try {
    lifecyclePreview.value = usesNativeStorage
      ? await previewProjectLifecycle(id)
      : {
          activeIssues: issues.items.filter((issue) => issue.project === name).length,
          nodes: 0,
          milestones: projects.items.find((project) => project.id === id)?.milestones.length ?? 0,
          runningTimers: 0,
          cycleDependencies: 0,
        };
  } catch (error) {
    lifecycleError.value = String(error);
  }
}

async function restoreProject(id: string) {
  try {
    await projects.setLifecycle(id, 'restore');
    lifecycleError.value = '';
  } catch (error) {
    lifecycleError.value = String(error);
  }
}

async function confirmLifecycle() {
  const request = lifecycleRequest.value;
  if (!request) return;
  try {
    await projects.setLifecycle(request.id, request.action);
    lifecycleRequest.value = null;
    lifecycleError.value = '';
  } catch (error) {
    lifecycleError.value = String(error);
  }
}

async function loadMoreProjects() {
  pagingBusy.value = true;
  pagingError.value = '';
  try {
    await projects.loadMore();
  } catch (error) {
    pagingError.value = String(error);
  } finally {
    pagingBusy.value = false;
  }
}

async function applyIndexQuery() {
  pagingError.value = '';
  try {
    await projects.setIndexQuery({
      sort: sort.value,
      status: statusFilter.value === 'All statuses' ? null : statusFilter.value,
      priority: priorityFilter.value === 'All priorities' ? null : priorityFilter.value,
      search: search.value.trim(),
    });
  } catch (error) {
    pagingError.value = String(error);
  }
}

watch([sort, statusFilter, priorityFilter], () => void applyIndexQuery());
watchDebounced(search, () => void applyIndexQuery(), { debounce: 150 });

const CreateProjectDialog = defineAsyncComponent(
  () => import('../../components/projects/CreateProjectDialog.vue'),
);

const rows = computed(() =>
  projects.items.map((project) => {
    // Native counts cover issues beyond the loaded issue page; local records serve preview mode.
    const projectIssues = issues.items.filter(
      (issue) => issue.project === project.name && !issue.triage,
    );
    const completed = projectIssues.filter(
      (issue) => issues.categoryFor(issue.status) === 'Completed',
    ).length;
    const total = issues.counts?.projects[project.name]?.total ?? projectIssues.length;
    const done = issues.counts?.projects[project.name]?.completed ?? completed;
    return {
      project,
      count: total,
      progress: total ? Math.round((done / total) * 100) : null,
    };
  }),
);

const columns = [
  { id: 'name', accessor: 'project', header: 'Name', size: 480 },
  {
    id: 'priority',
    accessor: (row: (typeof rows.value)[number]) => row.project.priority,
    header: 'Priority',
    size: 145,
  },
  {
    id: 'lead',
    accessor: (row: (typeof rows.value)[number]) =>
      row.project.leadActorId ? workspace.userName : '—',
    header: 'Lead',
    size: 145,
  },
  {
    id: 'targetDate',
    accessor: (row: (typeof rows.value)[number]) => row.project.targetDate ?? '—',
    header: 'Target date',
    size: 145,
  },
  { id: 'count', accessor: 'count', header: 'Issues', size: 95 },
  {
    id: 'status',
    accessor: (row: (typeof rows.value)[number]) => row.project.status,
    header: 'Status',
    size: 180,
  },
  { id: 'actions', accessor: 'project', header: '', size: 120 },
] as const;
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="flex min-h-11 items-center gap-2 px-3 py-1.5">
      <span
        class="rounded-md bg-(--ui-nav-active) px-2.5 py-1.5 text-xs font-medium text-foreground-highlighted"
      >
        All projects
      </span>
      <span class="text-xs tabular-nums text-foreground-dimmed">{{
        usesNativeStorage ? projects.totalCount : rows.length
      }}</span>
      <Button
        label="New project"
        icon="i-tabler-plus"
        size="sm"
        variant="ghost"
        class="ml-auto"
        @click="createOpen = true"
      />
      <Button
        :label="showInactive ? 'Active projects' : 'Archive and trash'"
        size="xs"
        variant="ghost"
        @click="showInactive = !showInactive"
      />
    </div>
    <div class="flex flex-wrap items-center gap-2 border-b border-outline px-3 py-2">
      <Input
        v-model="search"
        aria-label="Find projects"
        placeholder="Find projects"
        size="sm"
        class="w-48"
      />
      <Select
        v-model="statusFilter"
        :items="['All statuses', 'Backlog', 'Planned', 'In Progress', 'Completed', 'Canceled']"
        aria-label="Filter project status"
        size="sm"
      />
      <Select
        v-model="priorityFilter"
        :items="['All priorities', 'No priority', 'Low', 'Medium', 'High', 'Urgent']"
        aria-label="Filter project priority"
        size="sm"
      />
      <Select
        v-model="sort"
        :items="['updated', 'name', 'status', 'target', 'priority']"
        aria-label="Sort projects"
        size="sm"
      />
    </div>
    <div class="min-h-0 min-w-0 flex-1 overflow-auto">
      <div v-if="showInactive" class="px-3 py-2">
        <p v-if="lifecycleError" role="alert" class="mb-2 text-xs text-error">
          {{ lifecycleError }}
        </p>
        <p v-if="!projects.inactive.length" class="text-xs text-foreground-muted">
          No archived or trashed projects.
        </p>
        <div
          v-for="entry in projects.inactive"
          :key="entry.id"
          class="flex items-center gap-3 border-b border-outline px-2 py-2 text-xs"
        >
          <span class="min-w-0 flex-1 truncate">{{ entry.name }}</span>
          <span class="text-foreground-muted">{{
            entry.state === 'trash' ? 'Trash' : 'Archived'
          }}</span>
          <Button label="Restore" size="xs" variant="ghost" @click="restoreProject(entry.id)" />
        </div>
      </div>
      <div v-else-if="rows.length" class="w-full min-w-0 overflow-x-auto px-3">
        <DataTable
          :data="rows"
          :columns="columns"
          :row-key="(row) => row.project.id"
          :show-pagination="false"
          size="sm"
          aria-label="Workspace projects"
          :ui="{
            root: 'w-full min-w-0 border-0 bg-transparent text-xs',
            viewport: 'w-full overflow-x-auto',
            table: 'w-full! min-w-200 table-fixed',
            row: 'hover:bg-accented/50',
            headerCell: 'bg-transparent border-outline/60',
            cell: 'border-outline/60',
          }"
        >
          <template #name-cell="{ row }">
            <RouterLink
              :to="{
                name: 'project-overview',
                params: { project: row.original.project.name },
              }"
              class="flex min-w-0 items-center gap-2 rounded text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
            >
              <Icon
                :name="row.original.project.icon || defaultProjectIcon"
                :class="['size-4 shrink-0', iconColor(row.original.project.color).text]"
              />
              <span class="shrink-0 truncate font-medium">{{ row.original.project.name }}</span>
              <span
                v-if="row.original.project.summary"
                class="min-w-0 truncate text-foreground-muted"
                >{{ row.original.project.summary }}</span
              >
            </RouterLink>
          </template>
          <template #status-cell="{ row }">
            <span class="flex items-center gap-2 text-foreground-muted">
              <span class="size-1.5 rounded-full bg-warning" />
              {{ row.original.project.status }}
              <span class="tabular-nums text-foreground-dimmed">
                {{ row.original.progress === null ? '—' : `${row.original.progress}%` }}
              </span>
            </span>
          </template>
          <template #actions-cell="{ row }">
            <span class="flex items-center justify-end gap-1">
              <Button
                icon="i-tabler-archive"
                :aria-label="`Archive ${row.original.project.name}`"
                size="xs"
                variant="ghost"
                @click.stop="
                  requestLifecycle(row.original.project.id, row.original.project.name, 'archive')
                "
              />
              <Button
                icon="i-tabler-trash"
                :aria-label="`Move ${row.original.project.name} to trash`"
                size="xs"
                variant="ghost"
                @click.stop="
                  requestLifecycle(row.original.project.id, row.original.project.name, 'trash')
                "
              />
            </span>
          </template>
        </DataTable>
        <div v-if="projects.nextCursor" class="flex justify-center py-3">
          <Button
            label="Load more projects"
            size="sm"
            variant="ghost"
            :disabled="pagingBusy"
            @click="loadMoreProjects"
          />
        </div>
        <p v-if="pagingError" role="alert" class="px-2 py-2 text-xs text-error">
          {{ pagingError }}
        </p>
      </div>
      <div
        v-else
        class="flex h-full min-h-64 flex-col items-center justify-center gap-3 text-center"
      >
        <Icon name="i-tabler-cube" class="size-7 text-foreground-dimmed" />
        <p class="text-sm font-medium">No projects yet</p>
        <p class="text-xs text-foreground-muted">
          Create a project to organize related issues and milestones.
        </p>
        <Button label="Create project" icon="i-tabler-plus" size="sm" @click="createOpen = true" />
      </div>
    </div>
    <CreateProjectDialog v-if="createOpen" v-model:open="createOpen" />
    <Dialog
      :open="!!lifecycleRequest"
      :title="lifecycleRequest?.action === 'trash' ? 'Move project to trash' : 'Archive project'"
      size="sm"
      @update:open="if (!$event) lifecycleRequest = null;"
    >
      <template #body>
        <p class="text-sm text-foreground-muted">
          {{ lifecycleRequest?.name }} and its work can be restored later.
        </p>
        <p v-if="lifecyclePreview" class="mt-2 text-xs text-foreground-muted">
          {{ lifecyclePreview.activeIssues }} active issues
          {{ lifecycleRequest?.action === 'trash' ? 'will move to trash' : 'remain linked' }}.
        </p>
        <p
          v-if="lifecyclePreview && lifecycleRequest?.action === 'trash'"
          class="mt-1 text-xs text-foreground-muted"
        >
          {{ lifecyclePreview.nodes }} structure nodes and
          {{ lifecyclePreview.milestones }} milestones will move to trash.
        </p>
        <p v-if="lifecyclePreview?.runningTimers" class="mt-2 text-xs text-warning">
          {{ lifecycleRequest?.action === 'trash' ? 'Will pause' : 'Pause' }}
          {{ lifecyclePreview.runningTimers }} running timers{{
            lifecycleRequest?.action === 'trash' ? '.' : ' first.'
          }}
        </p>
        <p v-if="lifecyclePreview?.cycleDependencies" class="mt-2 text-xs text-warning">
          Resolve {{ lifecyclePreview.cycleDependencies }} cycle scopes or memberships first.
        </p>
        <p v-if="lifecycleError" role="alert" class="mt-2 text-xs text-error">
          {{ lifecycleError }}
        </p>
      </template>
      <template #footer>
        <Button label="Cancel" size="sm" variant="ghost" @click="lifecycleRequest = null" />
        <Button
          :label="lifecycleRequest?.action === 'trash' ? 'Move to trash' : 'Archive'"
          size="sm"
          :disabled="
            !lifecyclePreview ||
            (lifecycleRequest?.action === 'archive' && !!lifecyclePreview.runningTimers) ||
            !!lifecyclePreview.cycleDependencies
          "
          @click="confirmLifecycle"
        />
      </template>
    </Dialog>
  </div>
</template>
