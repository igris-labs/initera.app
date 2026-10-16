<script setup lang="ts">
import { computed, ref } from 'vue';
import { statusIcons, statusOrder } from '../../lib/issue-presentation';
import type { StatusColor } from '../../lib/issue-presentation';
import type { WorkflowCategory } from '../../types/workflow';
import { useIssuesStore } from '../../stores/issues';
import WorkflowGroupFields from '../issues/WorkflowGroupFields.vue';
import IssueStatusIcon from '../issues/IssueStatusIcon.vue';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';

const issues = useIssuesStore();
const editorOpen = ref(false);
const editingName = ref<string | null>(null);
const name = ref('');
const category = ref<WorkflowCategory>('Started');
const description = ref('');
const icon = ref(statusIcons[0] ?? 'i-tabler-circle');
const color = ref<StatusColor>('gray');
const error = ref('');

const activeStatuses = computed(() => issues.statusGroups);
const archivedStatuses = computed(() => issues.archivedStatusGroups);

function openEditor(status?: string) {
  editingName.value = status ?? null;
  name.value = status ?? '';
  category.value = status ? issues.categoryFor(status) : 'Started';
  description.value = status ? (issues.statusMeta[status]?.description ?? '') : '';
  icon.value = status ? issues.iconFor(status) : (statusIcons[0] ?? 'i-tabler-circle');
  color.value = status ? (issues.statusAppearance[status]?.color ?? 'gray') : 'gray';
  error.value = '';
  editorOpen.value = true;
}

function saveStatus() {
  const appearance = { icon: icon.value, color: color.value };
  const saved = editingName.value
    ? issues.customizeStatusGroup(
        editingName.value,
        name.value,
        appearance,
        category.value,
        description.value,
      )
    : issues.addStatusGroup(name.value, appearance, category.value, description.value);

  if (!saved) {
    error.value = 'Use a unique name of up to 32 characters and choose an icon and color.';
    return;
  }

  editorOpen.value = false;
}

function changeStatus(action: () => boolean) {
  error.value = action() ? '' : 'This status cannot be changed while it is in use or required.';
}
</script>

<template>
  <section id="workflow" aria-labelledby="workflow-heading">
    <div class="flex items-start justify-between gap-3">
      <div>
        <h3 id="workflow-heading" class="text-sm font-medium text-foreground-highlighted">
          Workflow statuses
        </h3>
        <p class="mt-1 text-xs text-foreground-muted">
          Set the stages used by issue lists and boards in this workspace.
        </p>
      </div>
      <Button label="New status" icon="i-tabler-plus" size="sm" @click="openEditor()" />
    </div>

    <p v-if="error" role="alert" class="mt-2 text-xs text-error">{{ error }}</p>
    <div class="mt-3 divide-y divide-outline border-y border-outline">
      <div
        v-for="status in activeStatuses"
        :key="status"
        class="flex min-h-11 items-center gap-3 py-2 text-xs"
      >
        <IssueStatusIcon :status="status" />
        <span class="min-w-0 flex-1 truncate text-foreground">{{ status }}</span>
        <span class="text-foreground-muted">{{ issues.categoryFor(status) }}</span>
        <span v-if="issues.statusMeta[status]?.isDefault" class="text-foreground-muted">
          Default
        </span>
        <Button
          v-else-if="['Backlog', 'Unstarted'].includes(issues.categoryFor(status))"
          :label="`Make ${status} default`"
          variant="ghost"
          size="xs"
          @click="changeStatus(() => issues.setDefaultStatusGroup(status))"
        />
        <Button
          v-if="!statusOrder.includes(status)"
          :label="`Edit ${status}`"
          variant="ghost"
          size="xs"
          @click="openEditor(status)"
        />
        <Button
          v-if="!statusOrder.includes(status) && !issues.statusMeta[status]?.isDefault"
          :label="`Archive ${status}`"
          variant="ghost"
          size="xs"
          @click="changeStatus(() => issues.archiveStatusGroup(status))"
        />
      </div>
      <div
        v-for="status in archivedStatuses"
        :key="status"
        class="flex min-h-11 items-center gap-3 py-2 text-xs"
      >
        <IssueStatusIcon :status="status" />
        <span class="min-w-0 flex-1 truncate text-foreground-muted">{{ status }}</span>
        <span class="text-foreground-muted">Archived</span>
        <Button
          :label="`Restore ${status}`"
          variant="ghost"
          size="xs"
          @click="changeStatus(() => issues.restoreStatusGroup(status))"
        />
      </div>
    </div>
  </section>

  <Dialog
    v-model:open="editorOpen"
    :title="editingName ? `Edit ${editingName}` : 'Create status'"
    size="lg"
    class="workbench-surface"
  >
    <template #body>
      <form class="space-y-4" @submit.prevent="saveStatus">
        <WorkflowGroupFields
          v-model:name="name"
          v-model:category="category"
          v-model:description="description"
          v-model:icon="icon"
          v-model:color="color"
          :mode="editingName ? 'edit' : 'create'"
        />
        <p v-if="error" role="alert" class="text-xs text-error">{{ error }}</p>
        <button type="submit" class="sr-only">Save status</button>
      </form>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="editorOpen = false" />
      <Button
        :label="editingName ? 'Save status' : 'Create status'"
        size="sm"
        @click="saveStatus"
      />
    </template>
  </Dialog>
</template>
