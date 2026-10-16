<script setup lang="ts">
import { useEventListener } from '@vueuse/core';
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useProjectsStore } from '../../stores/projects';
import { projectPickerItems } from '../../lib/project-presentation';
import type { IssueDetailAction } from '../../lib/issue-detail-command';
import type { Issue, Priority, Status } from '../../types/issue';
import { useIssueDetailEditor } from '../../composables/useIssueDetailEditor';
import { useIssueParentPicker } from '../../composables/useIssueParentPicker';
import { useUiStore } from '../../stores/ui';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Select } from '../ui/select';
import { DatePicker } from '../ui/date-picker';
import { Icon } from '../ui/icon';
import LabelPicker from '../common/LabelPicker.vue';
import PriorityPicker from '../common/PriorityPicker.vue';
import EstimatePicker from '../common/EstimatePicker.vue';
import StatusPicker from '../common/StatusPicker.vue';
import CyclePicker from '../common/CyclePicker.vue';
import MilestonePicker from '../common/MilestonePicker.vue';
import { RichTextEditor } from '../editor';
import IssueActivity from './IssueActivity.vue';
import IssueRelations from './IssueRelations.vue';
import IssueSubissues from './IssueSubissues.vue';
import IssueTime from './IssueTime.vue';
import IssueProjectMoveDialog from './IssueProjectMoveDialog.vue';
import IssueTriageDecisionDialog from './IssueTriageDecisionDialog.vue';

interface Props {
  issue: Issue;
  expanded?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  close: [];
  expand: [];
  update: [patch: Partial<Issue>];
  archive: [];
  moved: [previousId: string, nextId: string];
}>();

const ui = useUiStore();
const router = useRouter();
const projectStore = useProjectsStore();

const detailRoot = ref<HTMLElement | null>(null);
const moveOpen = ref(false);
const moveDestination = ref('');
const triageDialogOpen = ref(false);
const triageDecision = ref<'accept' | 'decline'>('accept');

const {
  editing,
  saving,
  editError,
  editState,
  titleDraft,
  titleMarkdownDraft,
  descriptionDraft,
  descriptionDocumentDraft,
  startDateDraft,
  dueDateDraft,
  dueDateEdited,
  beginEdit,
  saveEdit,
  onEditBlur,
} = useIssueDetailEditor(computed(() => props.issue));

const { parentSearch, parentError, parentIssue, parentItems, changeParent } = useIssueParentPicker(
  computed(() => props.issue),
);

const labels = computed({
  get: () => props.issue.labels,
  set: (value: string[]) => emit('update', { labels: value }),
});

const projectItems = computed(() => projectPickerItems(projectStore.items));
const projectMilestones = computed(
  () =>
    projectStore.items.find((project) => project.name === props.issue.project)?.milestones ?? [],
);

function updateProject(value: string) {
  if (value === props.issue.project) return;
  moveDestination.value = value;
  moveOpen.value = true;
}

function afterTriageDecision() {
  if (router.currentRoute.value.path !== '/triage') return;

  void router.replace({
    name: 'issue',
    params: { id: props.issue.id },
    query: { from: '/issues' },
  });
}

function addSubIssue() {
  ui.createIssueDefaults = {
    parentId: props.issue.id,
    project: props.issue.project,
    milestoneId: props.issue.milestoneId ?? null,
    status: 'Todo',
  };
  ui.createIssueOpen = true;
}

function openRelatedIssue(id: string) {
  router.push({ name: props.expanded ? 'issue-full' : 'issue', params: { id } });
}

useEventListener(window, 'issue-detail-command', (event) => {
  const { issueId, action } = (
    event as CustomEvent<{
      issueId: string;
      action: IssueDetailAction;
    }>
  ).detail;

  if (issueId !== props.issue.id) return;

  if (action === 'edit') {
    if (!editing.value) beginEdit();
    return;
  }

  if (action === 'archive') {
    if (!editing.value) emit('archive');
    return;
  }

  if (action === 'subissue') {
    if (!editing.value) addSubIssue();
    return;
  }

  const selector = {
    parent: '[aria-label="Parent issue"]',
    status: '[aria-label="Status"]',
    priority: '[aria-label="Priority"]',
    estimate: '[aria-label="Estimate"]',
    relation: '[data-issue-relation-add]',
    project: '[aria-label="Project"]',
    labels: '[data-issue-label-picker] button',
  }[action];

  detailRoot.value?.querySelector<HTMLElement>(selector)?.click();
});
</script>

<template>
  <component
    :is="expanded ? 'section' : 'aside'"
    ref="detailRoot"
    :aria-label="expanded ? 'Full issue view' : 'Issue details'"
    :class="[
      'workbench-surface flex h-full min-h-0 w-full flex-col bg-default',
      expanded && 'flex-1',
    ]"
  >
    <div
      class="flex h-11 shrink-0 items-center justify-between border-b border-outline px-4 text-xs text-foreground-muted"
    >
      <div class="flex min-w-0 items-center gap-2">
        <Button
          v-if="expanded"
          icon="i-tabler-arrow-left"
          aria-label="Back to issue preview"
          variant="ghost"
          @click="emit('close')"
        />
        <span class="truncate">{{ issue.id }}</span>
      </div>
      <div class="flex items-center gap-1">
        <Button
          v-if="issue.triage && !editing"
          label="Accept"
          icon="i-tabler-check"
          variant="ghost"
          size="xs"
          aria-label="Accept from triage"
          @click="
            triageDecision = 'accept';
            triageDialogOpen = true;
          "
        />
        <Button
          v-if="issue.triage && !editing"
          icon="i-tabler-x"
          variant="ghost"
          size="xs"
          aria-label="Decline from triage"
          @click="
            triageDecision = 'decline';
            triageDialogOpen = true;
          "
        />
        <Button
          v-if="!editing"
          icon="i-tabler-archive"
          aria-label="Archive issue"
          title="Archive issue"
          variant="ghost"
          @click="emit('archive')"
        />
        <Button
          v-if="!editing"
          icon="i-tabler-pencil"
          aria-label="Edit issue"
          title="Edit issue"
          variant="ghost"
          @click="beginEdit"
        />
        <Button
          v-if="!expanded"
          icon="i-tabler-maximize"
          aria-label="Open full issue view"
          title="Open full issue view"
          variant="ghost"
          @click="emit('expand')"
        />
        <Button
          v-if="!expanded"
          icon="i-tabler-x"
          aria-label="Close issue details"
          color="neutral"
          variant="ghost"
          @click="emit('close')"
        />
      </div>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto">
      <div
        :class="
          expanded
            ? 'mx-auto grid w-full max-w-6xl grid-cols-1 gap-x-12 gap-y-8 px-6 py-10 md:px-10 lg:grid-cols-[minmax(0,1fr)_16rem] lg:py-14'
            : 'space-y-5 px-4 py-5'
        "
      >
        <section :class="expanded ? 'min-w-0 lg:col-start-1' : 'space-y-2 px-1'">
          <button
            v-if="parentIssue && !editing"
            type="button"
            class="mb-3 flex items-center gap-1 text-xs text-foreground-muted hover:text-foreground"
            @click="openRelatedIssue(parentIssue.id)"
          >
            <Icon name="i-tabler-git-branch" class="size-3.5" />
            {{ parentIssue.id }} · {{ parentIssue.title }}
          </button>
          <form
            v-if="editing"
            class="space-y-3"
            @submit.prevent="saveEdit(true)"
            @focusout="onEditBlur"
          >
            <RichTextEditor
              v-model="titleMarkdownDraft"
              v-model:plain-text="titleDraft"
              mode="title"
              aria-label="Edit issue title"
              placeholder="Issue title"
            />
            <RichTextEditor
              v-model="descriptionDraft"
              v-model:document="descriptionDocumentDraft"
              aria-label="Edit issue description"
              placeholder="Add description..."
            />
            <div class="flex flex-wrap items-center gap-1.5">
              <DatePicker
                v-model="startDateDraft"
                label="Start date"
                calendar-label="Choose issue start date"
              />
              <DatePicker
                :model-value="dueDateDraft"
                :label="issue.due ?? 'Due date'"
                calendar-label="Choose issue due date"
                @update:model-value="
                  dueDateDraft = $event;
                  dueDateEdited = true;
                "
              />
              <Button
                v-if="startDateDraft"
                label="Clear start date"
                variant="ghost"
                size="xs"
                @click="startDateDraft = undefined"
              />
              <Button
                v-if="dueDateDraft || issue.due"
                label="Clear due date"
                variant="ghost"
                size="xs"
                @click="
                  dueDateDraft = undefined;
                  dueDateEdited = true;
                "
              />
            </div>
            <p v-if="editError" role="alert" class="text-xs text-error">{{ editError }}</p>
            <div class="flex items-center gap-2">
              <Button label="Save changes" type="submit" size="sm" :disabled="saving" />
              <Button
                v-if="editState === 'failed'"
                label="Retry save"
                variant="ghost"
                size="sm"
                @click="saveEdit()"
              />
              <span class="text-xs text-foreground-dimmed" role="status">
                {{
                  editState === 'saving'
                    ? 'Saving…'
                    : editState === 'failed'
                      ? 'Not saved'
                      : 'Saved'
                }}
              </span>
            </div>
          </form>
          <template v-else>
            <RichTextEditor
              v-if="issue.titleMarkdown"
              :model-value="issue.titleMarkdown"
              mode="title"
              readonly
              aria-label="Issue title"
            />
            <h2
              v-else
              :class="
                expanded
                  ? 'text-2xl font-semibold tracking-tight text-foreground-highlighted'
                  : 'text-lg font-semibold tracking-tight text-foreground-highlighted'
              "
            >
              {{ issue.title }}
            </h2>
            <RichTextEditor
              v-if="issue.description || issue.descriptionDocument?.content?.content?.length"
              :model-value="issue.description"
              :document="issue.descriptionDocument"
              readonly
              aria-label="Issue description"
              class="text-foreground-muted"
            />
            <p
              v-else
              :class="
                expanded
                  ? 'mt-5 max-w-prose whitespace-pre-wrap text-sm leading-6 text-foreground-muted'
                  : 'whitespace-pre-wrap text-sm leading-5 text-foreground-muted'
              "
            >
              Add a description when this issue needs more context.
            </p>
          </template>
          <IssueSubissues
            v-if="!editing"
            :issue="issue"
            @add="addSubIssue"
            @open="openRelatedIssue"
          />
          <IssueRelations v-if="!editing" :issue-id="issue.id" @open-issue="openRelatedIssue" />
        </section>
        <section
          :class="
            expanded
              ? 'min-w-0 lg:col-start-2 lg:row-span-2 lg:row-start-1'
              : 'border-t border-outline pt-4'
          "
        >
          <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Properties</h3>
          <dl class="grid grid-cols-[5.5rem_minmax(0,1fr)] items-center gap-y-1 text-xs">
            <dt class="px-1 text-foreground-muted">Status</dt>
            <dd class="min-w-0">
              <StatusPicker
                :model-value="issue.status"
                scope="issue"
                aria-label="Status"
                variant="ghost"
                size="sm"
                searchable
                search-placeholder="Change status..."
                class="w-full"
                @update:model-value="emit('update', { status: $event as Status })"
              />
            </dd>
            <dt class="px-1 text-foreground-muted">Priority</dt>
            <dd class="min-w-0">
              <PriorityPicker
                :model-value="issue.priority"
                aria-label="Priority"
                show-empty-label
                variant="ghost"
                size="sm"
                searchable
                search-placeholder="Set priority..."
                class="w-full"
                @update:model-value="emit('update', { priority: $event as Priority })"
              />
            </dd>
            <dt class="px-1 text-foreground-muted">Project</dt>
            <dd class="min-w-0">
              <Select
                :model-value="issue.project"
                :items="projectItems"
                aria-label="Project"
                :display-label="issue.project === 'Unassigned' ? 'Add to project' : undefined"
                placeholder-icon="i-tabler-cube"
                variant="ghost"
                size="sm"
                searchable
                search-placeholder="Move to project..."
                class="w-full"
                @update:model-value="updateProject"
              />
            </dd>
            <template v-if="projectMilestones.length">
              <dt class="px-1 text-foreground-muted">Milestone</dt>
              <dd class="min-w-0">
                <MilestonePicker
                  :model-value="issue.milestoneId ?? null"
                  :project="issue.project"
                  @update:model-value="emit('update', { milestoneId: $event })"
                />
              </dd>
            </template>
            <dt class="px-1 text-foreground-muted">Estimate</dt>
            <dd class="min-w-0">
              <EstimatePicker
                :model-value="issue.estimate"
                aria-label="Estimate"
                variant="ghost"
                size="sm"
                class="w-full"
                @update:model-value="emit('update', { estimate: $event })"
              />
            </dd>
            <dt class="px-1 text-foreground-muted">Cycle</dt>
            <dd class="min-w-0"><CyclePicker :issue-id="issue.id" /></dd>
            <dt class="px-1 text-foreground-muted">Parent</dt>
            <dd class="min-w-0">
              <Select
                :model-value="issue.parentId ?? 'none'"
                :items="parentItems"
                aria-label="Parent issue"
                variant="ghost"
                size="sm"
                searchable
                search-placeholder="Find a parent issue..."
                class="w-full"
                @update:search="parentSearch = $event"
                @update:model-value="changeParent"
              />
              <p v-if="parentError" role="alert" class="px-2 text-xs text-error">
                {{ parentError }}
              </p>
            </dd>
            <template v-if="issue.startDate">
              <dt class="px-1 text-foreground-muted">Start date</dt>
              <dd
                class="flex min-h-7 min-w-0 items-center gap-2 px-2.5 text-foreground-highlighted"
              >
                <Icon name="i-tabler-calendar" class="size-4 shrink-0" aria-hidden="true" />
                <span class="truncate">{{ issue.startDate }}</span>
              </dd>
            </template>
            <template v-if="issue.due">
              <dt class="px-1 text-foreground-muted">Due date</dt>
              <dd
                class="flex min-h-7 min-w-0 items-center gap-2 px-2.5 text-foreground-highlighted"
              >
                <Icon name="i-tabler-calendar" class="size-4 shrink-0" aria-hidden="true" />
                <span class="truncate">{{ issue.due }}</span>
              </dd>
            </template>
          </dl>
          <div class="mt-7">
            <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Labels</h3>
            <div class="flex flex-wrap items-center gap-1 px-1">
              <Badge
                v-for="label in issue.labels"
                :key="label"
                :label="label"
                dot
                variant="outline"
              />
              <span data-issue-label-picker><LabelPicker v-model="labels" compact /></span>
            </div>
          </div>
          <IssueTime v-if="usesNativeStorage" :issue-id="issue.id" />
        </section>
        <IssueActivity :issue-id="issue.id" :expanded="expanded" />
      </div>
    </div>
  </component>
  <IssueProjectMoveDialog
    v-model:open="moveOpen"
    :identifier="issue.id"
    :destination="moveDestination"
    @moved="(previousId, nextId) => emit('moved', previousId, nextId)"
  />
  <IssueTriageDecisionDialog
    v-model:open="triageDialogOpen"
    :issues="[issue]"
    :decision="triageDecision"
    @completed="afterTriageDecision"
  />
</template>
