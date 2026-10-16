<script setup lang="ts">
import { ref } from 'vue';
import { useLabelsStore } from '../../stores/labels';
import { notifications, notifySuccess } from '../../lib/notifications';
import type { LabelRecord } from '../../lib/label-storage';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Dialog } from '../ui/dialog';

const labels = useLabelsStore();
const labelEditorOpen = ref(false);
const labelName = ref('');
const labelColorValue = ref('#999aa3');
const editingLabel = ref<LabelRecord | null>(null);
const trashTarget = ref<LabelRecord | null>(null);
const labelError = ref('');
function openLabelEditor(label: LabelRecord | null) {
  editingLabel.value = label;
  labelName.value = label?.name ?? '';
  labelColorValue.value = label?.color.startsWith('#') ? label.color : '#999aa3';
  labelError.value = '';
  labelEditorOpen.value = true;
}

async function saveLabel() {
  try {
    const name = labelName.value.trim();
    if (editingLabel.value)
      await labels.update(editingLabel.value.id, labelName.value, labelColorValue.value);
    else await labels.create(labelName.value, labelColorValue.value);
    labelEditorOpen.value = false;
    notifySuccess(editingLabel.value ? 'Label updated' : 'Label created', name);
  } catch (error) {
    labelError.value = String(error);
  }
}

async function changeLabel(action: () => Promise<unknown>, message?: string) {
  labelError.value = '';
  try {
    await action();
    if (message) notifySuccess(message);
  } catch (error) {
    labelError.value = String(error);
  }
}

async function confirmTrashLabel() {
  if (!trashTarget.value) return;
  await changeLabel(
    () => labels.trash(trashTarget.value!.id),
    `Label ${trashTarget.value.name} trashed`,
  );
  if (!labelError.value) trashTarget.value = null;
}
</script>

<template>
  <section id="labels" aria-labelledby="labels-heading">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h3 id="labels-heading" class="text-sm font-medium text-foreground-highlighted">Labels</h3>
        <p class="mt-1 text-xs text-foreground-muted">
          Shared by issues and projects in this workspace.
        </p>
      </div>
      <Button label="New label" icon="i-tabler-plus" size="sm" @click="openLabelEditor(null)" />
    </div>
    <p v-if="labelError" role="alert" class="mt-2 text-xs text-error">{{ labelError }}</p>
    <div class="mt-3 divide-y divide-outline border-y border-outline">
      <div
        v-for="label in labels.items"
        :key="label.id"
        class="flex min-h-11 items-center gap-3 py-2 text-xs"
      >
        <span class="size-2.5 shrink-0 rounded-full" :style="{ backgroundColor: label.color }" />
        <span class="min-w-0 flex-1 truncate text-foreground">{{ label.name }}</span>
        <span v-if="label.deletedAt" class="text-foreground-muted">Trashed</span>
        <span v-else-if="label.archivedAt" class="text-foreground-muted">Archived</span>
        <span v-else class="text-foreground-muted"
          >{{ label.issueCount + label.projectCount }} uses</span
        >
        <Button
          v-if="!label.deletedAt"
          label="Edit"
          :aria-label="`Edit ${label.name}`"
          variant="ghost"
          size="xs"
          @click="openLabelEditor(label)"
        />
        <Button
          v-if="!label.deletedAt && !label.archivedAt"
          label="Archive"
          :aria-label="`Archive ${label.name}`"
          variant="ghost"
          size="xs"
          @click="changeLabel(() => labels.archive(label.id), `Label ${label.name} archived`)"
        />
        <Button
          v-else-if="label.archivedAt"
          label="Restore"
          :aria-label="`Restore ${label.name}`"
          variant="ghost"
          size="xs"
          @click="
            changeLabel(() => labels.restoreArchived(label.id), `Label ${label.name} restored`)
          "
        />
        <Button
          v-if="!label.deletedAt"
          label="Trash"
          :aria-label="`Trash ${label.name}`"
          variant="ghost"
          size="xs"
          @click="trashTarget = label"
        />
        <Button
          v-else
          label="Restore"
          :aria-label="`Restore ${label.name}`"
          variant="ghost"
          size="xs"
          @click="
            changeLabel(async () => {
              const report = await labels.restoreTrashed(label.id);
              notifications.add({
                title: 'Label restored',
                description: `${report.restored} associations restored, ${report.skipped} skipped.`,
                type: 'success',
              });
            })
          "
        />
      </div>
    </div>
  </section>
  <Dialog
    v-model:open="labelEditorOpen"
    :title="editingLabel ? 'Edit label' : 'Create label'"
    size="sm"
    class="workbench-surface"
  >
    <template #body>
      <div class="space-y-3">
        <Input
          v-model="labelName"
          aria-label="Label name"
          placeholder="Label name"
          maxlength="80"
          size="sm"
        />
        <label class="flex items-center gap-3 text-xs text-foreground-muted">
          Color
          <input
            v-model="labelColorValue"
            type="color"
            aria-label="Label color"
            class="size-8 cursor-pointer rounded border border-outline bg-transparent"
          />
          <span>{{ labelColorValue }}</span>
        </label>
        <p v-if="labelError" role="alert" class="text-xs text-error">{{ labelError }}</p>
      </div>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="labelEditorOpen = false" />
      <Button :label="editingLabel ? 'Save label' : 'Create label'" size="sm" @click="saveLabel" />
    </template>
  </Dialog>
  <Dialog
    :open="!!trashTarget"
    :title="`Trash ${trashTarget?.name ?? 'label'}?`"
    size="sm"
    class="workbench-surface"
    @update:open="
      (open: boolean) => {
        if (!open) trashTarget = null;
      }
    "
  >
    <template #body>
      <p class="text-sm text-foreground-muted">
        Remove this label from
        {{ (trashTarget?.issueCount ?? 0) + (trashTarget?.projectCount ?? 0) }} issues and projects.
        Associations can be restored while their owners still exist.
      </p>
      <p v-if="labelError" role="alert" class="mt-2 text-xs text-error">{{ labelError }}</p>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="trashTarget = null" />
      <Button label="Trash label" color="error" size="sm" @click="confirmTrashLabel" />
    </template>
  </Dialog>
</template>
