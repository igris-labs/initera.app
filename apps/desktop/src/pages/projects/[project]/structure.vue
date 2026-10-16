<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import ProjectTabs from '../../../components/projects/ProjectTabs.vue';
import { Button } from '../../../components/ui/button';
import { Dialog } from '../../../components/ui/dialog';
import { Icon } from '../../../components/ui/icon';
import { Input } from '../../../components/ui/input';
import { Select } from '../../../components/ui/select';
import { Textarea } from '../../../components/ui/textarea';
import type { PlanningNode } from '../../../lib/planning-storage';
import { getIssueStatuses, listIssueRecords, queryIssueIdPage } from '../../../lib/issue-storage';
import { savedViewQuery } from '../../../lib/saved-view-query';
import { usesNativeStorage } from '../../../lib/workspace-storage';
import type { Issue } from '../../../types/issue';
import { useIssuesStore } from '../../../stores/issues';
import { usePlanningStore } from '../../../stores/planning';
import { useProjectsStore } from '../../../stores/projects';

const route = useRoute();
const projects = useProjectsStore();
const issues = useIssuesStore();
const planning = usePlanningStore();

watch(
  () => String(route.params.project),
  (name) => void projects.ensureByName(name).catch(() => {}),
  { immediate: true },
);

const project = computed(() =>
  projects.items.find((item) => item.name === String(route.params.project)),
);
const nodes = computed(() => planning.items.filter((node) => node.projectId === project.value?.id));
const activeNodes = computed(() =>
  nodes.value.filter((node) => !node.archivedAt && !node.deletedAt),
);
const archivedNodes = computed(() =>
  nodes.value.filter((node) => node.archivedAt && !node.deletedAt),
);
const dissolvedNodes = computed(() => nodes.value.filter((node) => node.deletedAt));
const projectIssues = computed(() =>
  issues.items.filter((issue) => issue.project === project.value?.name && !issue.triage),
);
const nodeOpen = ref(false);
const assignOpen = ref(false);
const deleteOpen = ref(false);
const editing = ref<PlanningNode | null>(null);
const assigning = ref<PlanningNode | null>(null);
const deleting = ref<PlanningNode | null>(null);
const removeMode = ref<'archive' | 'dissolve'>('archive');
const parentId = ref('root');
const kind = ref<PlanningNode['kind']>('module');
const name = ref('');
const description = ref('');
const search = ref('');
const error = ref('');
const saving = ref(false);
const nodeStatuses = ref<Record<string, string>>({});
const nativeMatches = ref<Issue[]>([]);
let statusRevision = 0;
let matchRevision = 0;

watch(
  project,
  (value) => {
    if (value) void planning.load(value.id).catch((cause) => (error.value = String(cause)));
  },
  { immediate: true },
);

watch(
  [nodes, project],
  async ([currentNodes, currentProject]) => {
    if (!usesNativeStorage || !currentProject) return;
    const revision = ++statusRevision;
    const ids = [...new Set(currentNodes.flatMap((node) => node.issueIds))];
    try {
      const pages = await Promise.all(
        Array.from({ length: Math.ceil(ids.length / 200) }, (_, page) =>
          getIssueStatuses(ids.slice(page * 200, (page + 1) * 200)),
        ),
      );
      if (revision === statusRevision) nodeStatuses.value = Object.assign({}, ...pages);
    } catch (cause) {
      if (revision === statusRevision) error.value = String(cause);
    }
  },
  { immediate: true },
);

watch([assignOpen, search, project], async ([visible, term, currentProject]) => {
  if (!usesNativeStorage || !visible || !currentProject) return;
  const revision = ++matchRevision;
  const query = savedViewQuery({
    id: '',
    name: '',
    description: '',
    layout: 'list',
    groupBy: 'status',
    scope: 'all',
    status: 'All statuses',
    project: currentProject.name,
    favorite: false,
  });
  try {
    const page = await queryIssueIdPage({ ...query, text: term.trim() }, 60);
    const records = await listIssueRecords(page.ids);
    if (revision === matchRevision) nativeMatches.value = records;
  } catch (cause) {
    if (revision === matchRevision) error.value = String(cause);
  }
});

const excludedParents = computed(() => {
  const excluded = new Set<string>();
  if (!editing.value) return excluded;
  excluded.add(editing.value.id);
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of nodes.value) {
      if (node.parentId && excluded.has(node.parentId) && !excluded.has(node.id)) {
        excluded.add(node.id);
        changed = true;
      }
    }
  }

  return excluded;
});

const parentOptions = computed(() => [
  { label: 'Project root', value: 'root' },
  ...activeNodes.value
    .filter((node) => !excludedParents.value.has(node.id))
    .map((node) => ({ label: node.name, value: node.id })),
]);

function descendants(node: PlanningNode): PlanningNode[] {
  return [
    node,
    ...activeNodes.value.filter((child) => child.parentId === node.id).flatMap(descendants),
  ];
}

const rows = computed(() => {
  const ordered: {
    node: PlanningNode;
    depth: number;
    direct: number;
    done: number;
    total: number;
  }[] = [];
  function visit(parent: string | null, depth: number) {
    for (const node of activeNodes.value
      .filter((item) => item.parentId === parent)
      .sort((a, b) => a.position - b.position)) {
      const direct = usesNativeStorage
        ? node.issueIds.filter((id) => id in nodeStatuses.value).length
        : projectIssues.value.filter((issue) => node.issueIds.includes(issue.id)).length;
      const ids = new Set(descendants(node).flatMap((item) => item.issueIds));
      const matching = projectIssues.value.filter((issue) => ids.has(issue.id));
      const statuses = [...ids]
        .map((id) => nodeStatuses.value[id])
        .filter((status) => status !== undefined);
      ordered.push({
        node,
        depth,
        direct,
        total: usesNativeStorage ? statuses.length : matching.length,
        done: usesNativeStorage
          ? statuses.filter((status) => issues.categoryFor(status) === 'Completed').length
          : matching.filter((issue) => issues.categoryFor(issue.status) === 'Completed').length,
      });
      visit(node.id, depth + 1);
    }
  }

  visit(null, 0);
  return ordered;
});

const filteredIssues = computed(() =>
  (usesNativeStorage ? nativeMatches.value : projectIssues.value).filter((issue) =>
    `${issue.id} ${issue.title}`.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
);
const directIssueCount = computed(() => {
  if (!usesNativeStorage) {
    return projectIssues.value.filter(
      (issue) => !activeNodes.value.some((node) => node.issueIds.includes(issue.id)),
    ).length;
  }

  const assigned = new Set(activeNodes.value.flatMap((node) => node.issueIds));
  return Math.max(
    0,
    (issues.counts?.projects[project.value?.name ?? '']?.total ?? 0) -
      [...assigned].filter((id) => id in nodeStatuses.value).length,
  );
});

function openNode(node: PlanningNode | null, parent: string | null = null) {
  editing.value = node;
  parentId.value = node?.parentId ?? parent ?? 'root';
  kind.value = node?.kind ?? 'module';
  name.value = node?.name ?? '';
  description.value = node?.description ?? '';
  error.value = '';
  nodeOpen.value = true;
}

async function saveNode() {
  if (!project.value) return;
  saving.value = true;
  error.value = '';
  try {
    if (editing.value)
      await planning.update({
        ...editing.value,
        parentId: parentId.value === 'root' ? null : parentId.value,
        kind: kind.value,
        name: name.value,
        description: description.value,
      });
    else
      await planning.create(
        project.value.id,
        parentId.value === 'root' ? null : parentId.value,
        kind.value,
        name.value,
        description.value,
      );
    nodeOpen.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}

async function removeNode() {
  if (!deleting.value) return;
  saving.value = true;
  error.value = '';
  try {
    if (removeMode.value === 'archive') await planning.archive(deleting.value);
    else await planning.dissolve(deleting.value);
    deleteOpen.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}

async function restoreNode(node: PlanningNode, memberships = false) {
  error.value = '';
  try {
    if (node.deletedAt) await planning.restoreDissolved(node, memberships);
    else await planning.restoreArchived(node);
  } catch (cause) {
    error.value = String(cause);
  }
}

function siblingIndex(node: PlanningNode) {
  return activeNodes.value
    .filter((item) => item.parentId === node.parentId)
    .sort((a, b) => a.position - b.position)
    .findIndex((item) => item.id === node.id);
}

function siblingCount(node: PlanningNode) {
  return activeNodes.value.filter((item) => item.parentId === node.parentId).length;
}

async function moveNode(node: PlanningNode, offset: number) {
  error.value = '';
  try {
    await planning.reorder(node, siblingIndex(node) + offset);
  } catch (cause) {
    error.value = String(cause);
  }
}

async function toggleIssue(identifier: string) {
  if (!project.value || !assigning.value) return;
  error.value = '';
  try {
    await planning.assign(
      project.value.id,
      identifier,
      assigning.value.issueIds.includes(identifier) ? null : assigning.value.id,
    );
    assigning.value = nodes.value.find((node) => node.id === assigning.value?.id) ?? null;
  } catch (cause) {
    error.value = String(cause);
  }
}
</script>

<template>
  <div v-if="project" class="flex min-h-0 flex-1 flex-col">
    <ProjectTabs :project="project.name" />
    <main class="min-h-0 flex-1 overflow-auto bg-default px-6 py-6" aria-label="Project structure">
      <div class="mx-auto max-w-4xl">
        <header class="mb-6 flex items-start justify-between gap-4">
          <div>
            <h1 class="text-base font-semibold">Structure</h1>
            <p class="mt-1 text-xs text-foreground-muted">
              Nest modules and features, then assign issues to any node.
            </p>
          </div>
          <Button label="Add node" icon="i-tabler-plus" size="sm" @click="openNode(null)" />
        </header>
        <p v-if="error && !nodeOpen && !deleteOpen" role="alert" class="mb-3 text-xs text-error">
          {{ error }}
        </p>
        <div v-if="!rows.length" class="rounded-lg border border-outline p-8 text-center">
          <Icon name="i-tabler-sitemap" class="mx-auto size-6 text-foreground-muted" />
          <p class="mt-3 text-sm">No modules or features yet</p>
          <p class="mt-1 text-xs text-foreground-muted">
            Your issues can stay directly on the project.
          </p>
        </div>
        <div v-else class="divide-y divide-outline border-y border-outline">
          <div
            v-for="row in rows"
            :key="row.node.id"
            class="flex min-h-14 items-center gap-3 py-2"
            :style="{ paddingLeft: `${row.depth * 24}px` }"
          >
            <Icon
              :name="row.node.kind === 'module' ? 'i-tabler-folders' : 'i-tabler-components'"
              class="size-4 shrink-0 text-foreground-muted"
            />
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium">{{ row.node.name }}</p>
              <p class="text-xs text-foreground-muted">
                {{ row.node.kind }} · {{ row.direct }} direct · {{ row.done }}/{{ row.total }} done
              </p>
            </div>
            <Button
              icon="i-tabler-chevron-up"
              :aria-label="`Move ${row.node.name} up`"
              variant="ghost"
              size="xs"
              :disabled="siblingIndex(row.node) === 0"
              @click="moveNode(row.node, -1)"
            />
            <Button
              icon="i-tabler-chevron-down"
              :aria-label="`Move ${row.node.name} down`"
              variant="ghost"
              size="xs"
              :disabled="siblingIndex(row.node) === siblingCount(row.node) - 1"
              @click="moveNode(row.node, 1)"
            />
            <Button
              label="Issues"
              variant="ghost"
              size="xs"
              @click="
                assigning = row.node;
                search = '';
                assignOpen = true;
              "
            />
            <Button
              icon="i-tabler-plus"
              :aria-label="`Add child to ${row.node.name}`"
              variant="ghost"
              size="xs"
              @click="openNode(null, row.node.id)"
            />
            <Button
              icon="i-tabler-pencil"
              :aria-label="`Edit ${row.node.name}`"
              variant="ghost"
              size="xs"
              @click="openNode(row.node)"
            />
            <Button
              icon="i-tabler-archive"
              :aria-label="`Remove ${row.node.name}`"
              variant="ghost"
              size="xs"
              @click="
                deleting = row.node;
                removeMode = 'archive';
                deleteOpen = true;
              "
            />
          </div>
        </div>
        <section v-if="archivedNodes.length" class="mt-6" aria-label="Archived structure">
          <h2 class="text-xs font-medium text-foreground-muted">Archived nodes</h2>
          <div
            v-for="node in archivedNodes"
            :key="node.id"
            class="mt-2 flex items-center gap-2 text-xs"
          >
            <span class="min-w-0 flex-1 truncate">{{ node.name }}</span>
            <Button
              :label="`Restore ${node.name}`"
              variant="ghost"
              size="xs"
              @click="restoreNode(node)"
            />
          </div>
        </section>
        <section v-if="dissolvedNodes.length" class="mt-6" aria-label="Dissolved structure">
          <h2 class="text-xs font-medium text-foreground-muted">Dissolved nodes</h2>
          <div
            v-for="node in dissolvedNodes"
            :key="node.id"
            class="mt-2 flex items-center gap-2 text-xs"
          >
            <span class="min-w-0 flex-1 truncate">{{ node.name }}</span>
            <Button
              :label="`Restore ${node.name}`"
              variant="ghost"
              size="xs"
              @click="restoreNode(node)"
            />
            <Button
              :label="`Restore ${node.name} memberships`"
              variant="ghost"
              size="xs"
              @click="restoreNode(node, true)"
            />
          </div>
        </section>
        <p
          v-if="projectIssues.length || directIssueCount"
          class="mt-6 text-xs text-foreground-muted"
        >
          {{ directIssueCount }}
          issues directly on this project.
        </p>
      </div>
    </main>

    <Dialog
      v-model:open="nodeOpen"
      :title="editing ? 'Edit node' : 'Add project node'"
      size="md"
      class="workbench-surface"
    >
      <template #body>
        <form id="planning-node-form" class="space-y-4" @submit.prevent="saveNode">
          <Input
            v-model="name"
            aria-label="Node name"
            placeholder="Module or feature name"
            maxlength="120"
            size="sm"
          />
          <Select
            v-model="kind"
            :items="[
              { label: 'Module', value: 'module' },
              { label: 'Feature', value: 'feature' },
            ]"
            aria-label="Node kind"
            size="sm"
          />
          <Select v-model="parentId" :items="parentOptions" aria-label="Parent node" size="sm" />
          <Textarea
            v-model="description"
            aria-label="Node description"
            placeholder="Description"
            maxlength="10000"
            size="sm"
          />
          <p v-if="error" role="alert" class="text-xs text-error">{{ error }}</p>
        </form>
      </template>
      <template #footer>
        <Button label="Cancel" variant="ghost" size="sm" @click="nodeOpen = false" />
        <Button
          :label="editing ? 'Save node' : 'Add node'"
          size="sm"
          :disabled="saving"
          @click="saveNode"
        />
      </template>
    </Dialog>

    <Dialog
      v-model:open="assignOpen"
      :title="`Issues in ${assigning?.name ?? 'node'}`"
      size="md"
      class="workbench-surface"
    >
      <template #body>
        <Input
          v-model="search"
          aria-label="Find project issue"
          placeholder="Find issue…"
          size="sm"
        />
        <div class="mt-3 max-h-72 overflow-auto">
          <button
            v-for="issue in filteredIssues"
            :key="issue.id"
            type="button"
            class="flex min-h-9 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-accented"
            :aria-pressed="assigning?.issueIds.includes(issue.id)"
            @click="toggleIssue(issue.id)"
          >
            <span class="shrink-0 text-foreground-muted">{{ issue.id }}</span>
            <span class="min-w-0 flex-1 truncate">{{ issue.title }}</span>
            <Icon
              v-if="assigning?.issueIds.includes(issue.id)"
              name="i-tabler-check"
              class="size-4 text-info"
            />
          </button>
          <p v-if="!filteredIssues.length" class="px-2 py-4 text-xs text-foreground-muted">
            No issues found.
          </p>
        </div>
        <p v-if="error" role="alert" class="mt-2 text-xs text-error">{{ error }}</p>
      </template>
      <template #footer><Button label="Done" size="sm" @click="assignOpen = false" /></template>
    </Dialog>

    <Dialog
      v-model:open="deleteOpen"
      :title="`Remove ${deleting?.name ?? 'node'}?`"
      size="sm"
      class="workbench-surface"
    >
      <template #body>
        <p class="text-sm text-foreground-muted">
          {{ deleting ? descendants(deleting).length - 1 : 0 }} child nodes and
          {{
            deleting
              ? descendants(deleting).reduce((count, node) => count + node.issueIds.length, 0)
              : 0
          }}
          assigned issues in this subtree.
        </p>
        <div class="mt-4 space-y-2">
          <button
            type="button"
            class="w-full rounded-lg border border-outline p-3 text-left text-xs hover:bg-accented"
            :class="removeMode === 'archive' ? 'bg-accented' : ''"
            :aria-pressed="removeMode === 'archive'"
            @click="removeMode = 'archive'"
          >
            <strong class="block text-foreground">Archive subtree</strong>
            <span class="text-foreground-muted"
              >Hide this node and its children. Restore them later with their issue
              assignments.</span
            >
          </button>
          <button
            type="button"
            class="w-full rounded-lg border border-outline p-3 text-left text-xs hover:bg-accented"
            :class="removeMode === 'dissolve' ? 'bg-accented' : ''"
            :aria-pressed="removeMode === 'dissolve'"
            @click="removeMode = 'dissolve'"
          >
            <strong class="block text-foreground">Dissolve this node</strong>
            <span class="text-foreground-muted"
              >Move direct issues and child nodes to its parent. Keep a restore record.</span
            >
          </button>
        </div>
        <p v-if="error" role="alert" class="mt-2 text-xs text-error">{{ error }}</p>
      </template>
      <template #footer>
        <Button label="Cancel" variant="ghost" size="sm" @click="deleteOpen = false" />
        <Button
          :label="removeMode === 'archive' ? 'Archive subtree' : 'Dissolve node'"
          size="sm"
          :disabled="saving"
          @click="removeNode"
        />
      </template>
    </Dialog>
  </div>
  <div v-else class="p-8 text-sm text-foreground-muted">Project not found.</div>
</template>
