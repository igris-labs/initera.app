<script setup lang="ts">
import type { MenuItem } from 'akaza-ui';
import { computed, ref } from 'vue';
import { labelSwatch, priorityColor } from '../../lib/issue-presentation';
import { useIssuesStore } from '../../stores/issues';
import { useLabelsStore } from '../../stores/labels';
import { useProjectsStore } from '../../stores/projects';
import { priorities } from '../../types/issue';
import { estimateLabel, estimateOptions } from '../../lib/estimates';
import { usePreferencesStore } from '../../stores/preferences';
import { projectPickerItems } from '../../lib/project-presentation';
import { copyIssueBranchName } from '../../lib/issue-branch';
import type { Estimate, Issue, Priority, Status } from '../../types/issue';
import { ContextMenu } from '../ui/context-menu';
import { Icon } from '../ui/icon';
import IssueProjectMoveDialog from './IssueProjectMoveDialog.vue';

interface Props {
  issue: Issue | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  open: [id: string];
  update: [id: string, patch: Partial<Issue>];
  archive: [id: string];
  moved: [previousId: string, nextId: string];
}>();

const moveOpen = ref(false);
const moveDestination = ref('');
const moveIdentifier = ref('');

function proposeProjectMove(identifier: string, destination: string) {
  moveIdentifier.value = identifier;
  moveDestination.value = destination;
  moveOpen.value = true;
}

const issues = useIssuesStore();
const labels = useLabelsStore();
const projectStore = useProjectsStore();
const preferences = usePreferencesStore();

const labelOptions = computed(() =>
  [
    ...new Set([...labels.assignable.map((label) => label.name), ...(props.issue?.labels ?? [])]),
  ].sort(),
);

function localDate(offset: number) {
  const date = new Date();
  date.setDate(date.getDate() + offset);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

const items = computed<MenuItem[][]>(() => {
  const issue = props.issue;
  if (!issue) return [];
  const update = (patch: Partial<Issue>) => emit('update', issue.id, patch);
  return [
    [
      { type: 'label', label: issue.id },
      {
        label: 'Open issue',
        value: 'open',
        slot: 'action',
        onSelect: () => emit('open', issue.id),
      },
      {
        label: 'Copy branch name',
        value: 'copy-branch',
        icon: 'i-tabler-git-branch',
        kbds: ['meta', 'shift', 'b'],
        onSelect: () => void copyIssueBranchName(issue),
      },
    ],
    [
      {
        label: `Status · ${issue.status}`,
        value: 'status',
        slot: 'property',
        children: issues.visibleStatusGroups.map((status) => ({
          label: status,
          value: `status:${status}`,
          slot: 'choice',
          icon: issues.iconFor(status),
          iconClass: issues.colorFor(status),
          selected: status === issue.status,
          onSelect: () => update({ status: status as Status }),
        })),
      },
      {
        label: `Priority · ${issue.priority}`,
        value: 'priority',
        slot: 'property',
        children: priorities.map((priority) => ({
          label: priority,
          value: `priority:${priority}`,
          slot: 'choice',
          icon: 'i-tabler-antenna-bars-5',
          iconClass: priorityColor[priority],
          selected: priority === issue.priority,
          onSelect: () => update({ priority: priority as Priority }),
        })),
      },
      {
        label: issue.estimate
          ? `Estimate · ${estimateLabel(issue.estimate, preferences.estimateScale)}`
          : 'Estimate',
        value: 'estimate',
        slot: 'property',
        children: [
          {
            label: 'No estimate',
            value: 'estimate:none',
            slot: 'choice',
            icon: 'i-tabler-minus',
            selected: issue.estimate == null,
            onSelect: () => update({ estimate: null }),
          },
          ...estimateOptions(preferences.estimateScale).map((option) => ({
            label: option.label,
            value: `estimate:${option.value}`,
            slot: 'choice',
            icon: 'i-tabler-hash',
            selected: option.value === issue.estimate,
            onSelect: () => update({ estimate: option.value as Estimate }),
          })),
        ],
      },
      {
        label: `Project · ${issue.project}`,
        value: 'project',
        slot: 'property',
        children: projectPickerItems(projectStore.items).map((project) => ({
          label: project.label,
          value: `project:${project.value}`,
          slot: 'choice',
          icon: project.icon,
          iconClass: project.iconClass,
          selected: project.value === issue.project,
          onSelect: () => proposeProjectMove(issue.id, project.value),
        })),
      },
      {
        label: issue.labels.length ? `Labels · ${issue.labels.length}` : 'Labels',
        value: 'labels',
        slot: 'property',
        children: labelOptions.value.map((label) => ({
          label,
          value: `label:${label}`,
          type: 'checkbox' as const,
          slot: 'label-choice',
          checked: issue.labels.includes(label),
          onUpdateChecked: (checked: boolean) =>
            update({
              labels: checked
                ? [...issue.labels, label]
                : issue.labels.filter((current) => current !== label),
            }),
        })),
      },
      {
        label: issue.due ? `Due date · ${issue.due}` : 'Due date',
        value: 'due',
        slot: 'property',
        children: [
          {
            label: 'Today',
            value: 'due:today',
            slot: 'choice',
            icon: 'i-tabler-calendar',
            onSelect: () => update({ due: localDate(0) }),
          },
          {
            label: 'Tomorrow',
            value: 'due:tomorrow',
            slot: 'choice',
            icon: 'i-tabler-calendar',
            onSelect: () => update({ due: localDate(1) }),
          },
          {
            label: 'In 7 days',
            value: 'due:week',
            slot: 'choice',
            icon: 'i-tabler-calendar-time',
            onSelect: () => update({ due: localDate(7) }),
          },
          {
            label: 'Remove due date',
            value: 'due:none',
            slot: 'choice',
            icon: 'i-tabler-x',
            disabled: !issue.due,
            onSelect: () => update({ due: undefined }),
          },
        ],
      },
    ],
    [
      ...(issue.triage
        ? [
            {
              label: 'Accept from triage',
              value: 'accept-triage',
              icon: 'i-tabler-check',
              onSelect: () =>
                update({
                  triage: false,
                  status: issues.categoryFor(issue.status) === 'Backlog' ? 'Todo' : issue.status,
                }),
            },
            {
              label: 'Decline from triage',
              value: 'decline-triage',
              icon: 'i-tabler-x',
              onSelect: () => update({ triage: false, status: 'Canceled' }),
            },
          ]
        : issue.project === 'Unassigned'
          ? [
              {
                label: 'Send to triage',
                value: 'send-to-triage',
                icon: 'i-tabler-inbox',
                onSelect: () => update({ triage: true, status: 'Backlog' }),
              },
            ]
          : []),
      {
        label: 'Archive issue',
        value: 'archive',
        icon: 'i-tabler-archive',
        onSelect: () => emit('archive', issue.id),
      },
    ],
  ];
});
</script>

<template>
  <ContextMenu :items="items" class="workbench-surface w-56">
    <slot />
    <template #action="{ item }">
      <span class="flex w-full items-center gap-2">
        <Icon name="i-tabler-arrow-left" class="size-3.5 rotate-180 text-foreground-muted" />
        <span class="flex-1">{{ item.label }}</span>
      </span>
    </template>
    <template #property="{ item }">
      <span class="sr-only">{{ item.label }}</span>
      <span aria-hidden="true" class="flex w-full min-w-0 items-center gap-2">
        <Icon
          :name="
            {
              status: issue ? issues.iconFor(issue.status) : 'i-tabler-circle-dashed',
              priority: 'i-tabler-antenna-bars-5',
              estimate: 'i-tabler-hash',
              project: 'i-tabler-cube',
              labels: 'i-tabler-tag',
              due: 'i-tabler-calendar',
            }[String(item.value)] || 'i-tabler-dots'
          "
          :class="[
            'size-3.5 shrink-0 text-foreground-muted',
            item.value === 'status' && issue ? issues.colorFor(issue.status) : '',
            item.value === 'priority' && issue ? priorityColor[issue.priority] : '',
          ]"
        />
        <span class="shrink-0">{{
          {
            status: 'Status',
            priority: 'Priority',
            estimate: 'Estimate',
            project: 'Project',
            labels: 'Labels',
            due: 'Due date',
          }[String(item.value)]
        }}</span>
        <span class="ml-auto truncate text-xs text-foreground-muted">{{
          item.value === 'status'
            ? issue?.status
            : item.value === 'priority'
              ? issue?.priority
              : item.value === 'estimate'
                ? issue?.estimate == null
                  ? ''
                  : estimateLabel(issue.estimate, preferences.estimateScale)
                : item.value === 'project'
                  ? issue?.project
                  : item.value === 'labels'
                    ? issue?.labels.length === 1
                      ? issue.labels[0]
                      : issue?.labels.length || ''
                    : issue?.due
        }}</span>
        <Icon name="i-tabler-chevron-right" class="size-3 shrink-0 text-foreground-dimmed" />
      </span>
    </template>
    <template #choice="{ item }">
      <span class="flex w-full items-center gap-2">
        <Icon
          :name="String(item.icon)"
          :class="['size-3.5 shrink-0 text-foreground-muted', item.iconClass]"
        />
        <span class="flex-1">{{ item.label }}</span>
        <Icon v-if="item.selected" name="i-tabler-check" class="size-3.5 text-info" />
      </span>
    </template>
    <template #label-choice="{ item, checked }">
      <span class="flex w-full items-center gap-2">
        <span
          class="size-2.5 shrink-0 rounded-full"
          :style="{ backgroundColor: labelSwatch(String(item.label)) }"
        />
        <span class="flex-1">{{ item.label }}</span>
        <Icon v-if="checked" name="i-tabler-check" class="size-3.5 text-info" />
      </span>
    </template>
  </ContextMenu>
  <IssueProjectMoveDialog
    v-model:open="moveOpen"
    :identifier="moveIdentifier"
    :destination="moveDestination"
    @moved="(previousId, nextId) => emit('moved', previousId, nextId)"
  />
</template>
