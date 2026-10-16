<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { notifications } from '../../lib/notifications';
import { useViewsStore } from '../../stores/views';
import type { SavedView } from '../../types/view';
import type { IssueFilterTree } from '../../types/issue';
import FilterTreeEditor from './FilterTreeEditor.vue';
import { Button } from '../ui/button';
import { Checkbox } from '../ui/checkbox';
import { Dialog } from '../ui/dialog';
import { Input } from '../ui/input';
import { Select } from '../ui/select';
import { Textarea } from '../ui/textarea';

interface Props {
  view: SavedView;
}

const props = defineProps<Props>();

const open = defineModel<boolean>('open', { required: true });

const views = useViewsStore();
const router = useRouter();

const name = ref('');
const description = ref('');
const confirmingDelete = ref(false);
const filterTree = ref<IssueFilterTree>({ op: 'and', children: [] });
const columns = ref<string[]>([]);
const sortField = ref<NonNullable<SavedView['sortField']>>('position');
const sortDirection = ref<'asc' | 'desc'>('asc');
const density = ref<'compact' | 'default'>('default');
const showEmptyGroups = ref(false);
const includeSubIssues = ref(true);
const availableColumns = [
  'identifier',
  'title',
  'status',
  'priority',
  'project',
  'node',
  'milestone',
  'cycle',
  'labels',
  'estimate',
  'dueDate',
  'updatedAt',
  'trackedTime',
];

function toggleColumn(column: string, enabled: boolean) {
  columns.value = enabled
    ? [...columns.value, column]
    : columns.value.filter((item) => item !== column);
}

function syncFields(value: boolean) {
  if (!value) return;
  name.value = props.view.name;
  description.value = props.view.description;
  filterTree.value = structuredClone(props.view.filterTree ?? { op: 'and', children: [] });
  columns.value = [
    ...(props.view.columns ?? [
      'identifier',
      'title',
      'status',
      'priority',
      'project',
      'labels',
      'updatedAt',
    ]),
  ];
  sortField.value = props.view.sortField ?? 'position';
  sortDirection.value = props.view.sortDirection ?? 'asc';
  density.value = props.view.density ?? 'default';
  showEmptyGroups.value = props.view.showEmptyGroups ?? false;
  includeSubIssues.value = props.view.includeSubIssues ?? true;
  confirmingDelete.value = false;
}

watch(open, syncFields);

async function save() {
  if (!name.value.trim()) return;

  if (
    await views.update(props.view.id, {
      name: name.value.trim(),
      description: description.value.trim(),
      filterTree: filterTree.value,
      columns: columns.value,
      sortField: sortField.value,
      sortDirection: sortDirection.value,
      density: density.value,
      showEmptyGroups: showEmptyGroups.value,
      includeSubIssues: includeSubIssues.value,
    })
  )
    open.value = false;
}

async function duplicate() {
  try {
    const copy = await views.create(`${props.view.name} copy`);
    const saved = await views.update(copy.id, {
      ...props.view,
      id: copy.id,
      name: copy.name,
      favorite: false,
    });
    if (!saved) {
      await views.remove(copy.id);
      return;
    }

    open.value = false;
    await router.push({ name: 'saved-view', params: { viewId: copy.id } });
  } catch (error) {
    notifications.add({
      title: 'Could not duplicate view',
      description: String(error),
      type: 'error',
    });
  }
}

async function remove() {
  if (!confirmingDelete.value) {
    confirmingDelete.value = true;
    return;
  }

  try {
    await views.remove(props.view.id);
    open.value = false;
    await router.push({ name: 'views' });
  } catch (error) {
    notifications.add({
      title: 'Could not delete view',
      description: String(error),
      type: 'error',
    });
  }
}
</script>

<template>
  <Dialog
    v-model:open="open"
    title="Edit view"
    description="Update this local issue view."
    size="md"
    class="workbench-surface"
  >
    <template #body>
      <form id="view-settings-form" class="space-y-4" @submit.prevent="save">
        <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
          <span>Name</span>
          <Input v-model="name" aria-label="View name" class="w-full" />
        </label>
        <div class="space-y-2 border-t border-outline pt-4">
          <p class="text-xs font-medium text-foreground-muted">Filters</p>
          <FilterTreeEditor :node="filterTree" @change="filterTree = $event" />
        </div>
        <div class="space-y-2 border-t border-outline pt-4">
          <p class="text-xs font-medium text-foreground-muted">Visible columns</p>
          <div class="grid grid-cols-2 gap-2">
            <label
              v-for="column in availableColumns"
              :key="column"
              class="flex items-center gap-2 text-xs text-foreground"
            >
              <Checkbox
                :model-value="columns.includes(column)"
                :disabled="column === 'identifier' || column === 'title'"
                size="sm"
                @update:model-value="toggleColumn(column, Boolean($event))"
              />
              {{ column }}
            </label>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3 border-t border-outline pt-4">
          <label class="space-y-1.5 text-xs text-foreground-muted"
            >Sort field
            <Select
              v-model="sortField"
              :items="['position', 'title', 'status', 'priority', 'dueDate', 'updatedAt']"
              aria-label="Sort field"
            />
          </label>
          <label class="space-y-1.5 text-xs text-foreground-muted"
            >Direction
            <Select v-model="sortDirection" :items="['asc', 'desc']" aria-label="Sort direction" />
          </label>
          <label class="space-y-1.5 text-xs text-foreground-muted"
            >Density
            <Select v-model="density" :items="['default', 'compact']" aria-label="Row density" />
          </label>
          <div class="space-y-2 text-xs text-foreground-muted">
            <label class="flex items-center gap-2"
              ><Checkbox v-model="showEmptyGroups" size="sm" />Show empty groups</label
            >
            <label class="flex items-center gap-2"
              ><Checkbox v-model="includeSubIssues" size="sm" />Include sub-issues</label
            >
          </div>
        </div>
        <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
          <span>Description</span>
          <Textarea
            v-model="description"
            aria-label="View description"
            :rows="2"
            class="w-full resize-none"
          />
        </label>
      </form>
    </template>
    <template #footer>
      <Button
        :label="confirmingDelete ? 'Confirm delete' : 'Delete'"
        color="error"
        variant="ghost"
        class="mr-auto"
        @click="remove"
      />
      <Button label="Duplicate" variant="soft" @click="duplicate" />
      <Button label="Save" type="submit" form="view-settings-form" :disabled="!name.trim()" />
    </template>
  </Dialog>
</template>
