<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { useIssueBoardDrag } from '../../composables/useIssueBoardDrag';
import { priorityColor, statusIcons, statusOrder } from '../../lib/issue-presentation';
import type { StatusColor } from '../../lib/issue-presentation';
import type { StatusAppearance, StatusMeta, WorkflowCategory } from '../../types/workflow';
import type { Issue, Priority, Status } from '../../types/issue';
import { Icon } from '../ui/icon';
import { Button } from '../ui/button';
import { Popover } from '../ui/popover';
import IssueStatusIcon from './IssueStatusIcon.vue';
import IssueBoardCardContent from './IssueBoardCardContent.vue';
import IssueBoardVirtualCards from './IssueBoardVirtualCards.vue';
import WorkflowGroupFields from './WorkflowGroupFields.vue';

interface Props {
  items: Issue[];
  groupBy: 'none' | 'status' | 'priority' | 'project';
  showPriority: boolean;
  showProject: boolean;
  columns?: string[];
  density?: 'compact' | 'default';
  statusGroups: string[];
  archivedStatusGroups: string[];
  statusAppearance: Record<string, StatusAppearance>;
  statusMeta: Record<string, StatusMeta>;
  priorityGroups: Priority[];
  projectGroups: string[];
  statusUsage?: Record<string, number>;
  groupCounts?: Record<string, number>;
  loadingGroups?: Record<string, boolean>;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  move: [id: string, property: 'status' | 'priority', value: string, index: number];
  create: [property: 'status' | 'priority', value: string];
  addGroup: [
    name: string,
    icon: string,
    color: StatusColor,
    category: WorkflowCategory,
    description: string,
  ];
  editGroup: [
    name: string,
    nextName: string,
    icon: string,
    color: StatusColor,
    category: WorkflowCategory,
    description: string,
  ];
  archiveGroup: [name: string];
  restoreGroup: [name: string];
  deleteGroup: [name: string];
  reorderGroup: [property: 'status' | 'priority', name: string, index: number];
  setDefaultGroup: [name: string];
  loadMoreGroup: [name: string];
}>();

const selectedId = defineModel<string | null>('selectedId', { required: true });
const root = ref<HTMLElement | null>(null);
const addingGroup = ref(false);
const groupName = ref('');
const groupError = ref('');
const groupIcon = ref<string>(statusIcons[0]);
const groupColor = ref<StatusColor>('gray');
const groupCategory = ref<WorkflowCategory>('Started');
const groupDescription = ref('');
const editingGroup = ref<string | null>(null);
const editName = ref('');
const editIcon = ref<string>(statusIcons[0]);
const editColor = ref<StatusColor>('gray');
const editCategory = ref<WorkflowCategory>('Started');
const editDescription = ref('');
const groupNameInput = ref<{ focus: () => void } | null>(null);

const groups = computed(() =>
  (props.groupBy === 'status'
    ? [
        ...props.statusGroups,
        ...props.archivedStatusGroups.filter(
          (name) =>
            props.items.some((issue) => issue.status === name) ||
            (props.groupCounts?.[name] ?? 0) > 0,
        ),
      ]
    : props.groupBy === 'priority'
      ? props.priorityGroups
      : props.groupBy === 'project'
        ? props.projectGroups
        : ['All issues']
  ).map((name) => ({
    name,
    items: props.items.filter(
      (issue) =>
        props.groupBy === 'none' ||
        (props.groupBy === 'status' && issue.status === name) ||
        (props.groupBy === 'priority' && issue.priority === name) ||
        (props.groupBy === 'project' && issue.project === name),
    ),
  })),
);

const largeBoard = computed(() => props.items.length > 60);

const { canOpenIssue, moveWithKeyboard, moveGroupWithKeyboard } = useIssueBoardDrag({
  root,
  groups,
  groupBy: () => props.groupBy,
  archivedGroups: () => props.archivedStatusGroups,
  move: (id, property, value, index) => emit('move', id, property, value, index),
  reorderGroup: (property, name, index) => emit('reorderGroup', property, name, index),
});

function addGroup() {
  const name = groupName.value.trim();
  if (
    !name ||
    name.length > 32 ||
    [...props.statusGroups, ...props.archivedStatusGroups].some(
      (group) => group.toLowerCase() === name.toLowerCase(),
    )
  ) {
    groupError.value = 'Enter a unique group name (up to 32 characters).';
    return;
  }

  emit(
    'addGroup',
    name,
    groupIcon.value,
    groupColor.value,
    groupCategory.value,
    groupDescription.value.trim(),
  );
  groupName.value = '';
  groupIcon.value = statusIcons[0];
  groupColor.value = 'gray';
  groupCategory.value = 'Started';
  groupDescription.value = '';
  groupError.value = '';
  addingGroup.value = false;
}

function startEditing(name: string) {
  editingGroup.value = name;
  editName.value = name;
  editIcon.value = props.statusAppearance[name]?.icon ?? statusIcons[0];
  editColor.value = props.statusAppearance[name]?.color ?? 'gray';
  editCategory.value = props.statusMeta[name]?.category ?? 'Started';
  editDescription.value = props.statusMeta[name]?.description ?? '';
  groupError.value = '';
}

function saveGroup() {
  const name = editingGroup.value;
  const nextName = editName.value.trim();
  if (
    !name ||
    !nextName ||
    nextName.length > 32 ||
    [...props.statusGroups, ...props.archivedStatusGroups].some(
      (group) => group !== name && group.toLowerCase() === nextName.toLowerCase(),
    )
  ) {
    groupError.value = 'Enter a unique group name (up to 32 characters).';
    return;
  }

  emit(
    'editGroup',
    name,
    nextName,
    editIcon.value,
    editColor.value,
    editCategory.value,
    editDescription.value.trim(),
  );
  editingGroup.value = null;
  groupError.value = '';
}

function toggleArchive(name: string) {
  if (props.archivedStatusGroups.includes(name)) emit('restoreGroup', name);
  else emit('archiveGroup', name);
  editingGroup.value = null;
}

async function startAddingGroup() {
  addingGroup.value = true;
  await nextTick();
  groupNameInput.value?.focus();
}

function openIssue(id: string) {
  if (canOpenIssue()) selectedId.value = id;
}
</script>

<template>
  <div ref="root" class="flex min-h-full w-max min-w-full gap-3 p-3" aria-label="Issue board">
    <section
      v-for="group in groups"
      :key="group.name"
      :data-board-group="archivedStatusGroups.includes(group.name) ? undefined : group.name"
      class="flex w-70 shrink-0 flex-col"
      :aria-label="`${group.name} lane`"
    >
      <div
        class="mb-2 flex min-h-8 items-center gap-2 px-1 text-xs font-medium text-foreground-highlighted"
      >
        <div
          :data-board-group-handle="
            groupBy === 'project' || groupBy === 'none' || archivedStatusGroups.includes(group.name)
              ? undefined
              : ''
          "
          :role="
            groupBy === 'project' || groupBy === 'none' || archivedStatusGroups.includes(group.name)
              ? undefined
              : 'button'
          "
          :tabindex="
            groupBy === 'project' || groupBy === 'none' || archivedStatusGroups.includes(group.name)
              ? undefined
              : 0
          "
          :aria-label="
            groupBy === 'project' || groupBy === 'none' || archivedStatusGroups.includes(group.name)
              ? undefined
              : `Move ${group.name} group`
          "
          :aria-keyshortcuts="
            groupBy === 'project' || groupBy === 'none' || archivedStatusGroups.includes(group.name)
              ? undefined
              : 'Alt+ArrowLeft Alt+ArrowRight'
          "
          :class="[
            'flex min-w-0 flex-1 items-center gap-2 rounded-md px-1 py-1 focus-visible:outline-2 focus-visible:outline-info',
            archivedStatusGroups.includes(group.name) ? '' : 'cursor-default',
          ]"
          @keydown="moveGroupWithKeyboard($event, group.name)"
        >
          <IssueStatusIcon v-if="groupBy === 'status'" :status="group.name as Status" />
          <Icon
            v-else-if="groupBy === 'priority'"
            name="i-tabler-antenna-bars-5"
            :class="['size-4', priorityColor[group.name as Priority]]"
          />
          <Icon
            v-else
            :name="groupBy === 'project' ? 'i-tabler-cube' : 'i-tabler-list'"
            class="size-4 text-foreground-muted"
          />
          <span>{{ group.name }}</span>
          <span class="tabular-nums font-normal text-foreground-muted">{{
            groupCounts?.[group.name] ?? group.items.length
          }}</span>
          <span
            v-if="archivedStatusGroups.includes(group.name)"
            class="font-normal text-foreground-dimmed"
            >Archived</span
          >
        </div>
        <Popover
          v-if="groupBy === 'status'"
          :open="editingGroup === group.name"
          side="bottom"
          align="end"
          teleport="body"
          size="sm"
          class="w-56"
          @update:open="($event) => ($event ? startEditing(group.name) : (editingGroup = null))"
        >
          <template #trigger="{ toggle, triggerProps }">
            <button
              v-bind="triggerProps"
              type="button"
              :aria-label="`Edit ${group.name} group`"
              class="flex size-6 items-center justify-center rounded-md text-foreground-muted hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
              @click="toggle"
            >
              <Icon name="i-tabler-dots" class="size-4" />
            </button>
          </template>
          <template #content>
            <form class="space-y-3 p-2" @submit.prevent="saveGroup">
              <WorkflowGroupFields
                v-if="!statusOrder.includes(group.name)"
                v-model:name="editName"
                v-model:category="editCategory"
                v-model:description="editDescription"
                v-model:icon="editIcon"
                v-model:color="editColor"
                mode="edit"
              />
              <template v-else>
                <p class="px-1 text-xs font-medium text-foreground">{{ group.name }}</p>
                <p class="px-1 text-xs text-foreground-muted">{{ editCategory }}</p>
              </template>
              <p v-if="groupError" class="text-xs text-error">{{ groupError }}</p>
              <Button
                v-if="!statusOrder.includes(group.name)"
                label="Save group"
                type="submit"
                size="xs"
                class="w-full"
              />
              <button
                v-if="['Backlog', 'Unstarted'].includes(editCategory)"
                type="button"
                class="flex min-h-7 w-full items-center gap-2 rounded-md px-2 text-xs text-foreground-muted hover:bg-accented"
                :disabled="statusMeta[group.name]?.isDefault"
                @click="
                  emit('setDefaultGroup', group.name);
                  editingGroup = null;
                "
              >
                <Icon
                  :name="
                    statusMeta[group.name]?.isDefault ? 'i-tabler-check' : 'i-tabler-circle-check'
                  "
                  class="size-4"
                />
                {{ statusMeta[group.name]?.isDefault ? 'Default status' : 'Make default status' }}
              </button>
              <div v-if="!statusOrder.includes(group.name)" class="border-t border-outline pt-2">
                <button
                  type="button"
                  :disabled="statusMeta[group.name]?.isDefault"
                  :title="
                    statusMeta[group.name]?.isDefault
                      ? 'Choose another default status before archiving this group'
                      : undefined
                  "
                  class="flex min-h-7 w-full items-center gap-2 rounded-md px-2 text-xs text-foreground-muted hover:bg-accented hover:text-foreground-highlighted disabled:cursor-not-allowed disabled:opacity-40"
                  @click="toggleArchive(group.name)"
                >
                  <Icon
                    :name="
                      archivedStatusGroups.includes(group.name)
                        ? 'i-tabler-archive-off'
                        : 'i-tabler-archive'
                    "
                    class="size-4"
                  />
                  {{
                    archivedStatusGroups.includes(group.name) ? 'Restore group' : 'Archive group'
                  }}
                </button>
                <button
                  type="button"
                  :disabled="
                    (statusUsage?.[group.name] ?? group.items.length) > 0 ||
                    statusMeta[group.name]?.isDefault
                  "
                  :title="
                    statusMeta[group.name]?.isDefault
                      ? 'Choose another default status before deleting this group'
                      : (statusUsage?.[group.name] ?? group.items.length)
                        ? 'Move issues out of this group before deleting it'
                        : undefined
                  "
                  class="flex min-h-7 w-full items-center gap-2 rounded-md px-2 text-xs text-error hover:bg-accented disabled:cursor-not-allowed disabled:opacity-40"
                  @click="
                    emit('deleteGroup', group.name);
                    editingGroup = null;
                  "
                >
                  <Icon name="i-tabler-trash" class="size-4" /> Delete empty group
                </button>
                <p
                  v-if="statusUsage?.[group.name] ?? group.items.length"
                  class="px-2 pt-1 text-xs text-foreground-muted"
                >
                  Move its {{ statusUsage?.[group.name] ?? group.items.length }} issue{{
                    (statusUsage?.[group.name] ?? group.items.length) === 1 ? '' : 's'
                  }}
                  before deleting.
                </p>
              </div>
            </form>
          </template>
        </Popover>
      </div>
      <div class="min-h-28 flex-1 rounded-md bg-muted/35 p-1">
        <IssueBoardVirtualCards
          v-if="largeBoard"
          v-model:selected-id="selectedId"
          :items="group.items"
          :lane="group.name"
          :show-priority="showPriority"
          :show-project="showProject"
          :columns="columns"
          :density="density"
          :open-issue="openIssue"
          :move-with-keyboard="moveWithKeyboard"
        />
        <div v-else :data-board-lane="group.name" class="min-h-28 space-y-2">
          <div
            v-for="issue in group.items"
            :key="issue.id"
            :data-issue-id="issue.id"
            role="button"
            tabindex="0"
            :aria-label="issue.title"
            aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight"
            class="group cursor-default rounded-lg focus-visible:outline-2 focus-visible:outline-info"
            @click="openIssue(issue.id)"
            @keydown="moveWithKeyboard($event, issue)"
            @keydown.enter="selectedId = issue.id"
            @keydown.space.prevent="selectedId = issue.id"
          >
            <div
              :class="[
                'rounded-lg border border-outline bg-elevated transition-[border-color,background-color] duration-150 ease-out group-hover:border-outline/80 group-hover:bg-accented/50',
                density === 'compact' ? 'p-2' : 'p-3',
              ]"
            >
              <IssueBoardCardContent
                :issue="issue"
                :show-priority="showPriority"
                :show-project="showProject"
                :columns="columns"
                :density="density"
              />
            </div>
          </div>
        </div>
        <button
          v-if="(groupCounts?.[group.name] ?? group.items.length) > group.items.length"
          type="button"
          :disabled="loadingGroups?.[group.name]"
          class="mb-1 flex min-h-8 w-full items-center justify-center rounded-md text-xs text-foreground-muted hover:bg-accented hover:text-foreground-highlighted disabled:opacity-50"
          :aria-label="`Load more ${group.name} issues`"
          @click="emit('loadMoreGroup', group.name)"
        >
          {{
            loadingGroups?.[group.name]
              ? 'Loading…'
              : `Load more (${group.items.length} of ${groupCounts?.[group.name]})`
          }}
        </button>
        <button
          v-if="
            (groupBy === 'status' || groupBy === 'priority') &&
            !archivedStatusGroups.includes(group.name)
          "
          type="button"
          class="flex min-h-9 w-full items-center justify-center gap-1 rounded-lg border border-dashed border-outline/60 bg-elevated/40 text-xs text-foreground-muted transition-[background-color,border-color,color] duration-150 ease-out hover:border-outline hover:bg-elevated hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
          :aria-label="`New issue in ${group.name}`"
          @click="emit('create', groupBy, group.name)"
        >
          <Icon name="i-tabler-plus" class="size-3.5" /> New issue
        </button>
      </div>
    </section>
    <div v-if="groupBy === 'status'" class="w-70 shrink-0 pt-1">
      <form
        v-if="addingGroup"
        class="space-y-2"
        @submit.prevent="addGroup"
        @keydown.esc.prevent="addingGroup = false"
      >
        <WorkflowGroupFields
          ref="groupNameInput"
          v-model:name="groupName"
          v-model:category="groupCategory"
          v-model:description="groupDescription"
          v-model:icon="groupIcon"
          v-model:color="groupColor"
          mode="create"
          @update:name="groupError = ''"
        />
        <p v-if="groupError" class="text-xs text-error">{{ groupError }}</p>
        <div class="flex gap-1">
          <Button label="Add group" type="submit" size="xs" />
          <Button label="Cancel" variant="ghost" size="xs" @click="addingGroup = false" />
        </div>
      </form>
      <button
        v-else
        type="button"
        class="flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs text-foreground-muted transition-colors duration-150 ease-out hover:bg-muted/60 hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
        @click="startAddingGroup"
      >
        <Icon name="i-tabler-plus" class="size-4" /> Add group
      </button>
      <details v-if="archivedStatusGroups.length" class="mt-2 px-2 text-xs text-foreground-muted">
        <summary class="cursor-pointer rounded-md py-1 hover:text-foreground-highlighted">
          Archived groups ({{ archivedStatusGroups.length }})
        </summary>
        <div v-for="name in archivedStatusGroups" :key="name" class="mt-1 flex items-center gap-1">
          <span class="min-w-0 flex-1 truncate">{{ name }}</span>
          <button
            type="button"
            :aria-label="`Restore ${name}`"
            class="rounded-md px-1 py-1 hover:bg-accented"
            @click="emit('restoreGroup', name)"
          >
            Restore
          </button>
          <button
            v-if="
              (statusUsage?.[name] ?? items.filter((issue) => issue.status === name).length) === 0
            "
            type="button"
            :aria-label="`Delete ${name}`"
            class="rounded-md px-1 py-1 text-error hover:bg-accented"
            @click="emit('deleteGroup', name)"
          >
            Delete
          </button>
        </div>
      </details>
    </div>
  </div>
</template>
