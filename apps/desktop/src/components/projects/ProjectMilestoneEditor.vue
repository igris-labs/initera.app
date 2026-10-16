<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import type { CalendarDate } from '@internationalized/date';
import { nextTick, ref, watch } from 'vue';
import { previewMilestoneRemoval } from '../../lib/workspace-storage';
import type { MilestoneInput } from '../../lib/workspace-storage';
import { Button } from '../ui/button';
import { DatePicker } from '../ui/date-picker';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';

interface Props {
  projectId?: string;
}

const { projectId } = defineProps<Props>();
const milestones = defineModel<MilestoneInput[]>({ required: true });
const open = defineModel<boolean>('open', { required: true });

const editingIndex = ref<number | null>(null);
const title = ref('');
const description = ref('');
const target = ref<CalendarDate | undefined>();
const error = ref('');
const input = ref<{ inputRef?: HTMLInputElement } | null>(null);
const removeIndex = ref<number | null>(null);
const removeCount = ref(0);
const removeOpen = ref(false);

watch(open, (value) => {
  if (!value) resetForm();
});

function resetForm() {
  editingIndex.value = null;
  title.value = '';
  description.value = '';
  target.value = undefined;
  error.value = '';
}

function begin() {
  resetForm();
  open.value = true;
  nextTick(() => input.value?.inputRef?.focus());
}

function edit(index: number) {
  const milestone = milestones.value[index];
  if (!milestone) return;

  editingIndex.value = index;
  title.value = milestone.title;
  description.value = milestone.description;
  target.value = milestone.targetDate ? parseDate(milestone.targetDate) : undefined;
  open.value = true;
  nextTick(() => input.value?.inputRef?.focus());
}

function save() {
  const trimmedTitle = title.value.trim();
  if (!trimmedTitle || trimmedTitle.length > 240) {
    error.value = 'Enter a milestone title (up to 240 characters).';
    input.value?.inputRef?.focus();
    return;
  }

  const previous = editingIndex.value === null ? null : milestones.value[editingIndex.value];
  const milestone = {
    ...(previous?.id ? { id: previous.id } : {}),
    title: trimmedTitle,
    description: description.value.trim(),
    targetDate: target.value?.toString() ?? null,
  };

  if (editingIndex.value === null) milestones.value.push(milestone);
  else milestones.value.splice(editingIndex.value, 1, milestone);
  open.value = false;
}

function move(index: number, direction: -1 | 1) {
  const next = index + direction;
  if (next < 0 || next >= milestones.value.length) return;
  [milestones.value[index], milestones.value[next]] = [
    milestones.value[next]!,
    milestones.value[index]!,
  ];
}

async function requestRemoval(index: number) {
  const milestone = milestones.value[index];
  if (!milestone) return;
  if (!milestone.id || !projectId) {
    milestones.value.splice(index, 1);
    return;
  }

  try {
    removeCount.value = await previewMilestoneRemoval(projectId, milestone.id);
    removeIndex.value = index;
    removeOpen.value = true;
  } catch (cause) {
    error.value = String(cause);
  }
}

function confirmRemoval() {
  if (removeIndex.value !== null) milestones.value.splice(removeIndex.value, 1);
  removeOpen.value = false;
  removeIndex.value = null;
}
</script>

<template>
  <div class="mt-auto pt-5">
    <div class="rounded-lg border border-outline bg-default/40 p-2">
      <div v-if="milestones.length" class="space-y-1 pb-2">
        <div
          v-for="(milestone, index) in milestones"
          :key="index"
          class="flex items-center gap-2 rounded-md px-2 py-1 text-xs hover:bg-accented/50"
        >
          <Icon name="i-tabler-diamond" class="size-4 shrink-0 text-foreground-muted" />
          <button
            type="button"
            class="min-w-0 flex-1 truncate text-left text-foreground hover:underline"
            :aria-label="`Edit milestone ${milestone.title}`"
            @click="edit(index)"
          >
            {{ milestone.title }}
          </button>
          <span v-if="milestone.targetDate" class="text-foreground-muted">{{
            milestone.targetDate
          }}</span>
          <Button
            icon="i-tabler-arrow-up"
            variant="ghost"
            size="xs"
            :aria-label="`Move milestone ${milestone.title} up`"
            :disabled="index === 0"
            @click="move(index, -1)"
          />
          <Button
            icon="i-tabler-arrow-down"
            variant="ghost"
            size="xs"
            :aria-label="`Move milestone ${milestone.title} down`"
            :disabled="index === milestones.length - 1"
            @click="move(index, 1)"
          />
          <Button
            icon="i-tabler-x"
            variant="ghost"
            size="xs"
            :aria-label="`Remove milestone ${milestone.title}`"
            @click="requestRemoval(index)"
          />
        </div>
      </div>
      <div v-if="open" class="space-y-2 rounded-md border border-outline bg-elevated p-3">
        <div class="text-xs font-medium text-foreground-muted">
          {{ editingIndex === null ? 'Create milestone' : 'Edit milestone' }}
        </div>
        <div class="flex items-center gap-2">
          <Icon name="i-tabler-diamond" class="size-4 text-foreground-muted" />
          <Input
            ref="input"
            v-model="title"
            aria-label="Milestone name"
            placeholder="Milestone name"
            maxlength="240"
            size="sm"
            variant="bare"
            class="flex-1"
          />
        </div>
        <Textarea
          v-model="description"
          aria-label="Milestone description"
          placeholder="Add a description..."
          :rows="2"
          size="sm"
          variant="bare"
          class="min-h-12 resize-y"
        />
        <div class="flex flex-wrap items-center justify-between gap-2">
          <DatePicker
            v-model="target"
            label="Target date"
            calendar-label="Choose milestone target date"
          />
          <div class="flex items-center gap-1.5">
            <Button label="Cancel" variant="ghost" size="xs" @click="open = false" />
            <Button
              :label="editingIndex === null ? 'Add milestone' : 'Save milestone'"
              size="xs"
              @click="save"
            />
          </div>
        </div>
        <p v-if="error" role="alert" class="text-xs text-error">{{ error }}</p>
      </div>
      <button
        v-else
        type="button"
        class="flex min-h-8 w-full items-center justify-between rounded-md px-2 text-left text-xs text-foreground-muted hover:bg-accented hover:text-foreground"
        @click="begin"
      >
        <span>{{ milestones.length ? 'Add another milestone' : 'Milestones' }}</span>
        <Icon name="i-tabler-plus" class="size-4" />
      </button>
    </div>
  </div>

  <Dialog v-model:open="removeOpen" title="Remove milestone" size="sm">
    <template #body>
      <p class="text-xs text-foreground-muted">
        {{ removeCount }} issue{{ removeCount === 1 ? '' : 's' }} will lose this milestone when you
        save the project. You can restore compatible assignments later.
      </p>
    </template>
    <template #footer>
      <Button label="Cancel" size="sm" variant="ghost" @click="removeOpen = false" />
      <Button label="Remove milestone" size="sm" @click="confirmRemoval" />
    </template>
  </Dialog>
</template>
