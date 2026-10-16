<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import { useEventListener } from '@vueuse/core';
import type { CalendarDate } from '@internationalized/date';
import { computed, defineAsyncComponent, ref, watch } from 'vue';
import { RouterLink, useRoute } from 'vue-router';
import { RichTextEditor } from '../../../components/editor';
import LabelPicker from '../../../components/common/LabelPicker.vue';
import LeadPicker from '../../../components/common/LeadPicker.vue';
import IconPicker from '../../../components/common/IconPicker.vue';
import PriorityPicker from '../../../components/common/PriorityPicker.vue';
import StatusPicker from '../../../components/common/StatusPicker.vue';
import ProjectTabs from '../../../components/projects/ProjectTabs.vue';
import ProjectMilestones from '../../../components/projects/ProjectMilestones.vue';
import ProjectResources from '../../../components/projects/ProjectResources.vue';
import ProjectActivity from '../../../components/projects/ProjectActivity.vue';
import ProjectUpdateComposer from '../../../components/projects/ProjectUpdateComposer.vue';
import { Button } from '../../../components/ui/button';
import { Badge } from '../../../components/ui/badge';
import { DatePicker } from '../../../components/ui/date-picker';
import type {
  ProjectInput,
  ProjectPriority,
  ProjectRecord,
  ProjectStatus,
} from '../../../lib/workspace-storage';
import { changeProjectResource, usesNativeStorage } from '../../../lib/workspace-storage';
import { useIssuesStore } from '../../../stores/issues';
import { useProjectsStore } from '../../../stores/projects';
import { defaultProjectColor, defaultProjectIcon } from '../../../lib/icon-presentation';
import type { IconColor } from '../../../lib/icon-presentation';

const route = useRoute();
const projects = useProjectsStore();
const issues = useIssuesStore();

watch(
  () => String(route.params.project),
  (name) => void projects.ensureByName(name).catch(() => {}),
  { immediate: true },
);

const editOpen = ref(false);
const resourcesRef = ref<{ openCreate: () => void } | null>(null);
const propertyError = ref('');
const editingProjectId = ref<string | null>(null);
const CreateProjectDialog = defineAsyncComponent(
  () => import('../../../components/projects/CreateProjectDialog.vue'),
);

const project = computed(() => {
  const name = String(route.params.project);
  return (
    projects.items.find((item) => item.name === name) ??
    (editOpen.value ? projects.items.find((item) => item.id === editingProjectId.value) : undefined)
  );
});

const projectIssues = computed(() =>
  issues.items.filter((issue) => issue.project === project.value?.name && !issue.triage),
);

const doneCount = computed(
  () =>
    issues.counts?.projects[project.value?.name ?? '']?.completed ??
    projectIssues.value.filter((issue) => issues.categoryFor(issue.status) === 'Completed').length,
);
const totalCount = computed(
  () => issues.counts?.projects[project.value?.name ?? '']?.total ?? projectIssues.value.length,
);

const labels = computed({
  get: () => project.value?.labels ?? [],
  set: (value: string[]) => void updateProject({ labels: value }),
});

const projectIcon = computed({
  get: () => project.value?.icon || defaultProjectIcon,
  set: (value: string) => void updateProject({ icon: value }),
});

const projectColor = computed({
  get: () => project.value?.color || defaultProjectColor,
  set: (value: IconColor) => void updateProject({ color: value }),
});

const startDate = computed(() =>
  project.value?.startDate ? parseDate(project.value.startDate) : undefined,
);

const targetDate = computed(() =>
  project.value?.targetDate ? parseDate(project.value.targetDate) : undefined,
);

function editProject() {
  editingProjectId.value = project.value?.id ?? null;
  editOpen.value = true;
}

useEventListener(window, 'project-command', (event) => {
  const name = (event as CustomEvent<string>).detail;
  if (name === 'rename' || name === 'milestone') {
    editProject();
    return;
  }

  if (name === 'update') {
    document.querySelector<HTMLElement>('[aria-label="Project update composer"] button')?.click();
    return;
  }

  if (name === 'resource') {
    resourcesRef.value?.openCreate();
    return;
  }

  const selector =
    name === 'status'
      ? '[aria-label="Project status"]'
      : name === 'priority'
        ? '[aria-label="Project priority"]'
        : name === 'lead'
          ? '[aria-label="Project lead"]'
          : name === 'labels'
            ? '[data-project-label-picker] button'
            : name === 'startDate'
              ? '[aria-label="Choose project start date"]'
              : name === 'targetDate'
                ? '[aria-label="Choose project target date"]'
                : '';
  if (selector) document.querySelector<HTMLElement>(selector)?.click();
});

function projectInput(current: ProjectRecord): ProjectInput {
  return {
    name: current.name,
    nameMarkdown: current.nameMarkdown,
    identifierPrefix: current.identifierPrefix,
    icon: current.icon || defaultProjectIcon,
    color: current.color || defaultProjectColor,
    summary: current.summary,
    description: current.description,
    descriptionDocument: current.descriptionDocument,
    status: current.status,
    priority: current.priority,
    leadActorId: current.leadActorId ?? null,
    startDate: current.startDate,
    targetDate: current.targetDate,
    labels: current.labels ?? [],
    resources: current.resources ?? [],
    milestones: current.milestones,
  };
}

// Serialize property and resource writes against the same project version.
let writeQueue = Promise.resolve();

function updateResource(
  action: 'create' | 'update' | 'remove' | 'reorder',
  options: { resource?: ProjectRecord['resources'][number]; resourceId?: string; order?: string[] },
): Promise<void> {
  const write = writeQueue.then(async () => {
    const current = project.value;
    if (!current) return;

    if (!usesNativeStorage) {
      let resources = [...current.resources];
      if (action === 'create' && options.resource) resources.push(options.resource);

      if (action === 'update' && options.resource) {
        const resource = options.resource;
        resources = resources.map((item) => (item.id === resource.id ? resource : item));
      }

      if (action === 'remove')
        resources = resources.filter((item) => item.id !== options.resourceId);

      if (action === 'reorder' && options.order) {
        const byId = new Map(resources.map((item) => [item.id, item]));
        resources = options.order.flatMap((id) => byId.get(id) ?? []);
      }

      await projects.update(current.id, { ...projectInput(current), resources });
      return;
    }

    if (!current.version) throw new Error('Reload the project before changing resources.');

    await changeProjectResource(current.id, current.version, action, options);
    await projects.refreshOne(current.id);
  });
  writeQueue = write.catch(() => {});
  return write;
}

function updateProject(patch: Partial<ProjectInput>): Promise<void> {
  const write = writeQueue.then(async () => {
    const current = project.value;
    if (!current) return;
    await projects.update(current.id, { ...projectInput(current), ...patch });
    propertyError.value = '';
  });
  writeQueue = write.catch(() => {});
  return write.catch((cause) => {
    propertyError.value = String(cause);
  });
}

function updateDate(value: CalendarDate | undefined, field: 'startDate' | 'targetDate') {
  void updateProject({ [field]: value?.toString() ?? null });
}
</script>

<template>
  <div v-if="project" class="flex min-h-0 flex-1 flex-col">
    <ProjectTabs :project="project.name" />
    <div class="min-h-0 flex-1 overflow-auto">
      <div
        class="mx-auto grid w-full max-w-350 gap-8 px-5 py-7 lg:grid-cols-[minmax(0,1fr)_18rem] lg:px-10"
      >
        <main class="mx-auto w-full max-w-180 min-w-0 space-y-7">
          <section aria-label="Project overview">
            <div class="mb-4">
              <IconPicker
                v-model:icon="projectIcon"
                v-model:color="projectColor"
                aria-label="Choose project icon and color"
              />
            </div>
            <div class="flex items-start gap-3">
              <div class="min-w-0 flex-1">
                <RichTextEditor
                  v-if="project.nameMarkdown"
                  :model-value="project.nameMarkdown"
                  mode="title"
                  readonly
                  aria-label="Project name"
                  class="text-foreground-highlighted"
                />
                <h2 v-else class="text-xl font-semibold text-foreground-highlighted">
                  {{ project.name }}
                </h2>
                <RichTextEditor
                  v-if="project.summary"
                  :model-value="project.summary"
                  mode="summary"
                  readonly
                  aria-label="Project summary"
                  class="mt-1 text-foreground-muted"
                />
                <p v-else class="mt-1 text-xs text-foreground-dimmed">
                  Add a summary to give this project context.
                </p>
              </div>
              <Button
                icon="i-tabler-pencil"
                aria-label="Edit project"
                size="sm"
                variant="ghost"
                @click="editProject"
              />
            </div>
          </section>

          <ProjectResources
            ref="resourcesRef"
            :project="project"
            :update-resource="updateResource"
          />

          <section aria-label="Project description" class="border-t border-outline pt-6">
            <h3 class="mb-3 text-xs font-medium text-foreground-muted">Description</h3>
            <RichTextEditor
              v-if="project.description || project.descriptionDocument?.content?.content?.length"
              :model-value="project.description"
              :document="project.descriptionDocument"
              readonly
              aria-label="Project description"
            />
            <Button
              v-else
              label="Add a description"
              icon="i-tabler-plus"
              size="sm"
              variant="ghost"
              @click="editProject"
            />
          </section>

          <ProjectMilestones :project="project" @edit="editProject" />
          <ProjectUpdateComposer :project-id="project.id" />
          <ProjectActivity :project-id="project.id" :created-at="project.createdAt" />
        </main>

        <aside class="min-w-0 space-y-6" aria-label="Project details">
          <section aria-label="Project properties">
            <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Properties</h3>
            <dl class="grid grid-cols-[5.5rem_minmax(0,1fr)] items-center gap-y-1 text-xs">
              <dt class="px-1 text-foreground-muted">Status</dt>
              <dd class="min-w-0">
                <StatusPicker
                  scope="project"
                  :model-value="project.status"
                  aria-label="Project status"
                  variant="ghost"
                  size="sm"
                  class="w-full"
                  @update:model-value="updateProject({ status: $event as ProjectStatus })"
                />
              </dd>
              <dt class="px-1 text-foreground-muted">Priority</dt>
              <dd class="min-w-0">
                <PriorityPicker
                  :model-value="project.priority"
                  aria-label="Project priority"
                  show-empty-label
                  variant="ghost"
                  size="sm"
                  class="w-full"
                  @update:model-value="updateProject({ priority: $event as ProjectPriority })"
                />
              </dd>
              <dt class="px-1 text-foreground-muted">Lead</dt>
              <dd class="min-w-0">
                <LeadPicker
                  :model-value="project.leadActorId ?? null"
                  aria-label="Project lead"
                  variant="ghost"
                  size="sm"
                  class="w-full"
                  @update:model-value="updateProject({ leadActorId: $event })"
                />
              </dd>
              <dt class="px-1 text-foreground-muted">Start date</dt>
              <dd class="min-w-0">
                <DatePicker
                  :model-value="startDate"
                  label="Set start date"
                  calendar-label="Choose project start date"
                  variant="ghost"
                  @update:model-value="updateDate($event, 'startDate')"
                />
              </dd>
              <dt class="px-1 text-foreground-muted">Target date</dt>
              <dd class="min-w-0">
                <DatePicker
                  :model-value="targetDate"
                  label="Set target date"
                  calendar-label="Choose project target date"
                  variant="ghost"
                  @update:model-value="updateDate($event, 'targetDate')"
                />
              </dd>
            </dl>
            <p v-if="propertyError" role="alert" class="mt-2 px-1 text-xs text-error">
              {{ propertyError }}
            </p>
            <div class="mt-7">
              <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Labels</h3>
              <div class="flex flex-wrap items-center gap-1 px-1">
                <Badge
                  v-for="label in project.labels ?? []"
                  :key="label"
                  :label="label"
                  dot
                  variant="outline"
                />
                <span data-project-label-picker><LabelPicker v-model="labels" compact /></span>
              </div>
            </div>
          </section>
          <section class="border-t border-outline pt-4" aria-label="Project progress">
            <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Issues</h3>
            <p class="px-1 text-xs text-foreground-muted">
              {{ doneCount }} of {{ totalCount }} completed
            </p>
            <div class="mx-1 mt-2 h-1 overflow-hidden rounded-full bg-accented">
              <div
                class="h-full rounded-full bg-info transition-[width] duration-200"
                :style="{
                  width: `${totalCount ? (doneCount / totalCount) * 100 : 0}%`,
                }"
              />
            </div>
            <RouterLink
              :to="{
                name: 'project-issues',
                params: { project: project.name },
              }"
              class="mt-3 inline-flex rounded px-1 text-xs text-foreground-muted hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
              >View issues →</RouterLink
            >
          </section>
        </aside>
      </div>
    </div>
    <CreateProjectDialog v-if="editOpen" v-model:open="editOpen" :project="project" />
  </div>
  <div v-else class="p-8 text-sm text-foreground-muted">Project not found.</div>
</template>
