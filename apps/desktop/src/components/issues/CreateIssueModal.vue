<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import type { CalendarDate } from '@internationalized/date';
import { computed, nextTick, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useIssuesStore } from '../../stores/issues';
import { useUiStore } from '../../stores/ui';
import { useWorkspaceStore } from '../../stores/workspace';
import { useProjectsStore } from '../../stores/projects';
import { useRecoveryDraft } from '../../composables/useRecoveryDraft';
import { projectPickerItems } from '../../lib/project-presentation';
import { notifySuccess } from '../../lib/notifications';
import type { Estimate, Priority, Project, Status } from '../../types/issue';
import type { RichTextDocument } from '../../types/rich-text';
import { Button } from '../ui/button';
import { Select } from '../ui/select';
import { Dialog } from '../ui/dialog';
import { DatePicker } from '../ui/date-picker';
import { Icon } from '../ui/icon';
import { Switch } from '../ui/switch';
import { RichTextEditor } from '../editor';
import LabelPicker from '../common/LabelPicker.vue';
import PriorityPicker from '../common/PriorityPicker.vue';
import EstimatePicker from '../common/EstimatePicker.vue';
import StatusPicker from '../common/StatusPicker.vue';
import MilestonePicker from '../common/MilestonePicker.vue';

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

const router = useRouter();
const issues = useIssuesStore();
const ui = useUiStore();
const workspace = useWorkspaceStore();
const projectStore = useProjectsStore();

const title = ref('');
const titleMarkdown = ref('');
const description = ref('');
const descriptionDocument = ref<RichTextDocument | null>(null);
const project = ref<Project>('Unassigned');
const milestoneId = ref<string | null>(null);
const status = ref<Status>('Backlog');
const priority = ref<Priority>('No priority');
const estimate = ref<Estimate | null>(null);
const labels = ref<string[]>([]);
const parentId = ref<string | null>(null);
const startDate = ref<CalendarDate | undefined>();
const dueDate = ref<CalendarDate | undefined>();
const createMore = ref(false);
const error = ref('');
const titleInput = ref<{ focus: () => void } | null>(null);

interface IssueCreateDraft {
  title: string;
  titleMarkdown: string;
  description: string;
  descriptionDocument: RichTextDocument | null;
  project: Project;
  milestoneId: string | null;
  status: Status;
  priority: Priority;
  estimate: Estimate | null;
  labels: string[];
  parentId: string | null;
  startDate: string | null;
  dueDate: string | null;
}

const {
  value: recoveryDraft,
  waitUntilReady: waitForRecoveryDraft,
  discard: discardRecoveryDraft,
} = useRecoveryDraft<IssueCreateDraft>(
  computed(
    () => `issue-create:${workspace.workspaceId}:${ui.createIssueDefaults?.parentId ?? 'root'}`,
  ),
);

const currentDraft = computed<IssueCreateDraft>(() => ({
  title: title.value,
  titleMarkdown: titleMarkdown.value,
  description: description.value,
  descriptionDocument: descriptionDocument.value,
  project: project.value,
  milestoneId: milestoneId.value,
  status: status.value,
  priority: priority.value,
  estimate: estimate.value,
  labels: labels.value,
  parentId: parentId.value,
  startDate: startDate.value?.toString() ?? null,
  dueDate: dueDate.value?.toString() ?? null,
}));

let applyingRecovery = false;

watch(currentDraft, (draft) => {
  if (!open.value || applyingRecovery) return;
  recoveryDraft.value =
    draft.title || draft.description || draft.descriptionDocument ? draft : null;
});

watch(
  open,
  async (visible) => {
    if (!visible) return;
    await waitForRecoveryDraft();
    const draft = recoveryDraft.value;
    if (!draft || !open.value) return;
    applyingRecovery = true;
    title.value = draft.title;
    titleMarkdown.value = draft.titleMarkdown;
    description.value = draft.description;
    descriptionDocument.value = draft.descriptionDocument;
    project.value = draft.project;
    milestoneId.value = draft.milestoneId;
    status.value = draft.status;
    priority.value = draft.priority;
    estimate.value = draft.estimate;
    labels.value = [...draft.labels];
    parentId.value = draft.parentId;
    startDate.value = draft.startDate ? parseDate(draft.startDate) : undefined;
    dueDate.value = draft.dueDate ? parseDate(draft.dueDate) : undefined;
    await nextTick();
    applyingRecovery = false;
  },
  { immediate: true },
);

watch(dialogOpen, (value) => {
  if (!value && open.value) open.value = false;
  if (value) nextTick(() => titleInput.value?.focus());
});

const projectOptions = computed(() => projectPickerItems(projectStore.items));

function updateProject(value: string) {
  project.value = value;
  milestoneId.value = null;
}

watch(
  open,
  (value) => {
    if (value) {
      status.value = ui.createIssueDefaults?.status ?? issues.defaultStatus;
      priority.value = ui.createIssueDefaults?.priority ?? 'No priority';
      project.value = ui.createIssueDefaults?.project ?? 'Unassigned';
      milestoneId.value = ui.createIssueDefaults?.milestoneId ?? null;
      parentId.value = ui.createIssueDefaults?.parentId ?? null;
    } else ui.createIssueDefaults = null;
  },
  { immediate: true },
);

async function createIssue() {
  if (!title.value.trim()) {
    error.value = 'Enter an issue title.';
    titleInput.value?.focus();
    return;
  }

  if (startDate.value && dueDate.value && startDate.value.toString() > dueDate.value.toString()) {
    error.value = 'Due date must be on or after the start date.';
    return;
  }

  let issue;
  try {
    issue = await issues.create({
      title: title.value,
      titleMarkdown: titleMarkdown.value,
      description: description.value,
      descriptionDocument: descriptionDocument.value,
      project: project.value,
      milestoneId: milestoneId.value,
      status: status.value,
      priority: priority.value,
      estimate: estimate.value,
      labels: labels.value,
      parentId: parentId.value,
      startDate: startDate.value?.toString() ?? null,
      due: dueDate.value?.toString(),
    });
  } catch (cause) {
    error.value = String(cause);
    return;
  }

  notifySuccess(parentId.value ? 'Sub-issue created' : 'Issue created', issue.id);

  await discardRecoveryDraft();

  title.value = '';
  titleMarkdown.value = '';
  description.value = '';
  descriptionDocument.value = null;
  project.value = ui.createIssueDefaults?.project ?? 'Unassigned';
  milestoneId.value = ui.createIssueDefaults?.milestoneId ?? null;
  status.value = ui.createIssueDefaults?.status ?? issues.defaultStatus;
  priority.value = ui.createIssueDefaults?.priority ?? 'No priority';
  estimate.value = null;
  labels.value = [];
  startDate.value = undefined;
  dueDate.value = undefined;
  parentId.value = ui.createIssueDefaults?.parentId ?? null;
  error.value = '';

  if (createMore.value) {
    nextTick(() => titleInput.value?.focus());
    return;
  }

  const keepCurrentView = !!ui.createIssueDefaults;

  open.value = false;

  if (!keepCurrentView) router.push(issue.triage ? '/triage' : '/issues');
}
</script>

<template>
  <Dialog
    v-model:open="dialogOpen"
    :title="parentId ? 'Create sub-issue' : 'Create issue'"
    :description="`Capture work for ${workspace.workspaceName}.`"
    size="lg"
    class="workbench-surface"
    :ui="{
      content: 'max-w-2xl',
      header: 'min-h-11 border-b-0 px-4 pt-3 pb-0',
      body: 'px-4 pt-3 pb-3',
      footer: 'border-t-0 px-4 pt-0 pb-3',
    }"
  >
    <template #header="{ close, titleId }">
      <div class="flex min-w-0 items-center gap-2 text-xs text-foreground-muted">
        <h2 :id="titleId" class="sr-only">{{ parentId ? 'Create sub-issue' : 'Create issue' }}</h2>
        <span class="rounded-md bg-muted px-2 py-1 font-medium text-foreground">{{
          workspace.workspaceName
        }}</span>
        <Icon name="i-tabler-chevron-right" class="size-3.5" />
        <span>{{ parentId ? `New sub-issue of ${parentId}` : 'New issue' }}</span>
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
      <form id="create-issue-form" class="space-y-2" @submit.prevent="createIssue">
        <div>
          <RichTextEditor
            ref="titleInput"
            v-model="titleMarkdown"
            v-model:plain-text="title"
            mode="title"
            placeholder="Issue title"
            aria-label="Title"
            class="w-full"
            @update:model-value="error = ''"
          />
          <p v-if="error" id="issue-title-error" class="text-xs text-error">
            {{ error }}
          </p>
        </div>
        <div>
          <RichTextEditor
            v-model="description"
            v-model:document="descriptionDocument"
            placeholder="Add description..."
            aria-label="Description"
          />
        </div>
        <div class="flex flex-wrap items-center gap-1.5 pt-2">
          <StatusPicker
            id="issue-status"
            v-model="status"
            scope="issue"
            aria-label="Status"
            variant="pill"
            size="sm"
            side="bottom"
            searchable
            search-placeholder="Change status..."
          />
          <PriorityPicker
            id="issue-priority"
            v-model="priority"
            aria-label="Priority"
            variant="pill"
            size="sm"
            side="bottom"
            searchable
            search-placeholder="Set priority..."
          />
          <EstimatePicker
            v-model="estimate"
            aria-label="Estimate"
            variant="pill"
            size="sm"
            side="bottom"
          />
          <Select
            id="issue-project"
            :model-value="project"
            :items="projectOptions"
            aria-label="Project"
            variant="pill"
            size="sm"
            side="bottom"
            searchable
            search-placeholder="Move to project..."
            @update:model-value="updateProject"
          />
          <MilestonePicker v-model="milestoneId" :project="project" variant="pill" />
          <LabelPicker v-model="labels" pill />
          <DatePicker
            v-model="startDate"
            label="Start date"
            calendar-label="Choose issue start date"
          />
          <DatePicker v-model="dueDate" label="Due date" calendar-label="Choose issue due date" />
        </div>
      </form>
    </template>
    <template #footer>
      <label class="mr-auto flex items-center gap-2 text-xs text-foreground-muted">
        <Switch v-model="createMore" aria-label="Create more" size="sm" />Create more
      </label>
      <Button
        :label="parentId ? 'Create sub-issue' : 'Create issue'"
        type="submit"
        form="create-issue-form"
      />
    </template>
  </Dialog>
</template>
