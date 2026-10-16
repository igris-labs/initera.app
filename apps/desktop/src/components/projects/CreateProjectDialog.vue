<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import type { CalendarDate } from '@internationalized/date';
import { computed, nextTick, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import type {
  MilestoneInput,
  ProjectInput,
  ProjectPriority,
  ProjectRecord,
  ProjectStatus,
} from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import { useProjectsStore } from '../../stores/projects';
import { useWorkspaceStore } from '../../stores/workspace';
import { useRecoveryDraft } from '../../composables/useRecoveryDraft';
import { defaultProjectColor, defaultProjectIcon } from '../../lib/icon-presentation';
import { notifySuccess } from '../../lib/notifications';
import type { IconColor } from '../../lib/icon-presentation';
import type { RichTextDocument } from '../../types/rich-text';
import { RichTextEditor } from '../editor';
import LabelPicker from '../common/LabelPicker.vue';
import LeadPicker from '../common/LeadPicker.vue';
import IconPicker from '../common/IconPicker.vue';
import PriorityPicker from '../common/PriorityPicker.vue';
import StatusPicker from '../common/StatusPicker.vue';
import { Button } from '../ui/button';
import { DatePicker } from '../ui/date-picker';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import ProjectMilestoneEditor from './ProjectMilestoneEditor.vue';

interface Props {
  project?: ProjectRecord | null;
}

const props = defineProps<Props>();

const open = defineModel<boolean>('open', { required: true });
const dialogOpen = ref(false);

watch(
  open,
  (value) => {
    if (value)
      nextTick(() => {
        dialogOpen.value = true;
      });
    else dialogOpen.value = false;
  },
  { immediate: true },
);

watch(dialogOpen, (value) => {
  if (!value && open.value) open.value = false;
});

const router = useRouter();
const workspace = useWorkspaceStore();
const projects = useProjectsStore();
const issues = useIssuesStore();

const editing = computed(() => !!props.project);

const name = ref('');
const nameMarkdown = ref('');
const prefix = ref('');
const prefixEdited = ref(false);
const icon = ref(defaultProjectIcon);
const color = ref<IconColor>(defaultProjectColor);
const summary = ref('');
const description = ref('');
const descriptionDocument = ref<RichTextDocument | null>(null);
const status = ref<ProjectStatus>('Backlog');
const priority = ref<ProjectPriority>('No priority');
const leadActorId = ref<string | null>(null);
const startDate = ref<CalendarDate | undefined>();
const targetDate = ref<CalendarDate | undefined>();
const milestones = ref<MilestoneInput[]>([]);
const labels = ref<string[]>([]);
const resources = ref<ProjectInput['resources']>([]);
const milestoneOpen = ref(false);
const error = ref('');

const saving = ref(false);
const nameInput = ref<{ focus: () => void } | null>(null);

const {
  value: recoveryDraft,
  waitUntilReady: waitForRecoveryDraft,
  discard: discardRecoveryDraft,
} = useRecoveryDraft<ProjectInput>(computed(() => `project-create:${workspace.workspaceId}`));

const currentDraft = computed<ProjectInput>(() => ({
  name: name.value,
  nameMarkdown: nameMarkdown.value,
  identifierPrefix: prefix.value,
  icon: icon.value,
  color: color.value,
  summary: summary.value,
  description: description.value,
  descriptionDocument: descriptionDocument.value,
  status: status.value,
  priority: priority.value,
  leadActorId: leadActorId.value,
  startDate: startDate.value?.toString() ?? null,
  targetDate: targetDate.value?.toString() ?? null,
  milestones: milestones.value,
  labels: labels.value,
  resources: resources.value,
}));

let applyingRecovery = false;

watch(currentDraft, (draft) => {
  if (!open.value || editing.value || applyingRecovery) return;
  recoveryDraft.value =
    draft.name || draft.summary || draft.description || draft.descriptionDocument ? draft : null;
});

watch(
  open,
  async (visible) => {
    if (!visible || editing.value) return;
    await waitForRecoveryDraft();
    const draft = recoveryDraft.value;
    if (!draft || !open.value) return;
    applyingRecovery = true;
    name.value = draft.name;
    nameMarkdown.value = draft.nameMarkdown;
    prefix.value = draft.identifierPrefix;
    prefixEdited.value = true;
    icon.value = draft.icon;
    color.value = draft.color;
    summary.value = draft.summary;
    description.value = draft.description;
    descriptionDocument.value = draft.descriptionDocument ?? null;
    status.value = draft.status;
    priority.value = draft.priority;
    leadActorId.value = draft.leadActorId;
    startDate.value = draft.startDate ? parseDate(draft.startDate) : undefined;
    targetDate.value = draft.targetDate ? parseDate(draft.targetDate) : undefined;
    milestones.value = [...draft.milestones];
    labels.value = [...draft.labels];
    resources.value = [...draft.resources];
    await nextTick();
    applyingRecovery = false;
  },
  { immediate: true },
);
const suggestedPrefix = computed(() => {
  const words = name.value.trim().split(/\s+/).filter(Boolean);
  return (words.length > 1 ? words.map((word) => word[0]).join('') : (words[0]?.slice(0, 3) ?? ''))
    .replace(/[^a-z]/gi, '')
    .toUpperCase()
    .slice(0, 8);
});

watch(name, () => {
  if (!prefixEdited.value) prefix.value = suggestedPrefix.value;
});

watch(
  open,
  (value) => {
    if (value) {
      reset();
      if (props.project) {
        const project = props.project;
        name.value = project.name;
        nameMarkdown.value = project.nameMarkdown || project.name;
        prefix.value = project.identifierPrefix;
        icon.value = project.icon || defaultProjectIcon;
        color.value = project.color || defaultProjectColor;
        prefixEdited.value = true;
        summary.value = project.summary;
        description.value = project.description;
        descriptionDocument.value = project.descriptionDocument ?? null;
        status.value = project.status;
        priority.value = project.priority;
        leadActorId.value = project.leadActorId ?? null;
        startDate.value = project.startDate ? parseDate(project.startDate) : undefined;
        targetDate.value = project.targetDate ? parseDate(project.targetDate) : undefined;
        milestones.value = project.milestones.map((milestone) => ({
          ...milestone,
        }));
        labels.value = [...(project.labels ?? [])];
        resources.value = (project.resources ?? []).map((resource) => ({
          ...resource,
        }));
      }

      nextTick(() => nameInput.value?.focus());
    }
  },
  { immediate: true },
);

function dateString(date: CalendarDate | undefined): string | null {
  return date?.toString() ?? null;
}

function reset() {
  name.value = '';
  nameMarkdown.value = '';
  prefix.value = '';
  prefixEdited.value = false;
  icon.value = defaultProjectIcon;
  color.value = defaultProjectColor;
  summary.value = '';
  description.value = '';
  descriptionDocument.value = null;
  status.value = 'Backlog';
  priority.value = 'No priority';
  leadActorId.value = null;
  startDate.value = undefined;
  targetDate.value = undefined;
  milestones.value = [];
  labels.value = [];
  resources.value = [];
  milestoneOpen.value = false;
  error.value = '';
}

function editPrefix(value: string) {
  prefixEdited.value = true;
  prefix.value = value.toUpperCase();
}

async function submit() {
  if (summary.value.length > 500) {
    error.value = 'Keep the project summary under 500 characters.';
    return;
  }

  if (milestoneOpen.value) {
    error.value = 'Save or cancel the milestone before saving the project.';
    return;
  }

  const input: ProjectInput = {
    name: name.value,
    nameMarkdown: nameMarkdown.value,
    identifierPrefix: prefix.value,
    icon: icon.value,
    color: color.value,
    summary: summary.value,
    description: description.value,
    descriptionDocument: descriptionDocument.value,
    status: status.value,
    priority: priority.value,
    leadActorId: leadActorId.value,
    startDate: dateString(startDate.value),
    targetDate: dateString(targetDate.value),
    milestones: milestones.value,
    labels: labels.value,
    resources: resources.value,
  };

  saving.value = true;
  error.value = '';

  try {
    const previousName = props.project?.name;
    const project = props.project
      ? await projects.update(props.project.id, input)
      : await projects.create(input);

    if (previousName && previousName !== project.name)
      issues.items.forEach((issue) => {
        if (issue.project === previousName) issue.project = project.name;
      });

    if (!previousName) await discardRecoveryDraft();
    reset();
    open.value = false;
    router.push({
      name: 'project-overview',
      params: { project: project.name },
    });
    if (!previousName) notifySuccess('Project created', project.name);
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Dialog
    v-model:open="dialogOpen"
    :title="editing ? 'Edit project' : 'Create project'"
    :description="`${editing ? 'Edit' : 'Create'} a project in ${workspace.workspaceName}.`"
    size="lg"
    class="workbench-surface"
    :ui="{
      content: 'flex h-[min(86dvh,64rem)] w-[min(92vw,56rem)] max-w-none flex-col overflow-hidden',
      header: 'min-h-12 shrink-0 border-b-0 px-5 pt-4 pb-1',
      body: 'flex min-h-0 flex-1 flex-col overflow-y-auto px-5 pt-2 pb-3',
      footer: 'shrink-0 px-5 py-3',
    }"
  >
    <template #header="{ close, titleId }">
      <div class="flex min-w-0 items-center gap-2 text-xs text-foreground-muted">
        <h2 :id="titleId" class="sr-only">
          {{ editing ? 'Edit project' : 'Create project' }}
        </h2>
        <span
          class="rounded-md border border-outline bg-muted px-2 py-1 font-medium text-foreground"
          >{{ workspace.workspaceName }}</span
        >
        <Icon name="i-tabler-chevron-right" class="size-3.5" />
        <span>{{ editing ? 'Edit project' : 'New project' }}</span>
      </div>
      <Button
        icon="i-tabler-x"
        variant="ghost"
        size="icon"
        aria-label="Close dialog"
        @click="close"
      />
    </template>
    <template #body>
      <form id="create-project-form" class="flex min-h-0 flex-1 flex-col" @submit.prevent="submit">
        <div class="mb-3">
          <IconPicker
            v-model:icon="icon"
            v-model:color="color"
            aria-label="Choose project icon and color"
            size="sm"
          />
        </div>
        <RichTextEditor
          ref="nameInput"
          v-model="nameMarkdown"
          v-model:plain-text="name"
          mode="title"
          aria-label="Project name"
          placeholder="Project name"
        />
        <RichTextEditor
          v-model="summary"
          aria-label="Project summary"
          mode="summary"
          placeholder="Add a short summary..."
        />
        <div class="mt-2 flex flex-wrap items-center gap-1.5">
          <StatusPicker
            v-model="status"
            scope="project"
            aria-label="Project status"
            variant="pill"
            size="sm"
          />
          <PriorityPicker
            v-model="priority"
            aria-label="Project priority"
            variant="pill"
            size="sm"
          />
          <LeadPicker v-model="leadActorId" aria-label="Project lead" variant="pill" size="sm" />
          <DatePicker
            :model-value="startDate"
            label="Start date"
            calendar-label="Choose project start date"
            @update:model-value="startDate = $event"
          />
          <DatePicker
            :model-value="targetDate"
            label="Target date"
            calendar-label="Choose project target date"
            @update:model-value="targetDate = $event"
          />
          <LabelPicker v-model="labels" pill />
          <label
            class="flex min-h-7 items-center gap-1.5 rounded-lg border border-outline bg-default px-2.5 text-xs text-foreground-muted dark:bg-muted"
          >
            <span>Identifier</span>
            <Input
              :model-value="prefix"
              aria-label="Identifier (2–8 letters)"
              placeholder="APP"
              maxlength="8"
              variant="bare"
              size="sm"
              :ui="{
                root: 'min-h-0 w-14 border-0 bg-transparent p-0 text-xs font-semibold focus-visible:ring-0',
              }"
              @update:model-value="editPrefix($event)"
            />
          </label>
        </div>
        <div class="mt-4 border-t border-outline pt-6">
          <RichTextEditor
            v-model="description"
            v-model:document="descriptionDocument"
            aria-label="Project description"
            placeholder="Write a description, a project brief, or collect ideas..."
          />
        </div>
        <ProjectMilestoneEditor
          v-model="milestones"
          v-model:open="milestoneOpen"
          :project-id="project?.id"
        />
        <p v-if="error" role="alert" class="pt-2 text-xs text-error">
          {{ error }}
        </p>
      </form>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" class="mr-auto" @click="open = false" />
      <Button
        :label="editing ? 'Save project' : 'Create project'"
        type="submit"
        form="create-project-form"
        size="sm"
        :disabled="saving"
      />
    </template>
  </Dialog>
</template>
