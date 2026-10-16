<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { RichTextEditor } from '../editor';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { useProjectUpdatesStore } from '../../stores/project-updates';
import { notifySuccess } from '../../lib/notifications';

interface Props {
  projectId: string;
}

const props = defineProps<Props>();

const updates = useProjectUpdatesStore();

const open = ref(false);
const body = ref('');
const saving = ref(false);
const error = ref('');
const editor = ref<{ focus: () => void } | null>(null);

const hasUpdates = computed(() =>
  Boolean(
    updates.hasMore[props.projectId] ||
    updates.byProject[props.projectId]?.some((entry) => entry.kind !== 'change'),
  ),
);

async function openComposer() {
  open.value = true;
  await nextTick();
  editor.value?.focus();
}

async function postUpdate() {
  if (!body.value.trim()) {
    error.value = 'Write an update before posting.';
    return;
  }

  saving.value = true;
  error.value = '';
  try {
    await updates.create(props.projectId, body.value);
    notifySuccess('Project update posted');
    body.value = '';
    open.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <section aria-label="Project update composer">
    <button
      v-if="!open"
      type="button"
      class="flex w-full items-center justify-center gap-2 rounded-md border border-outline px-4 py-3 text-xs font-medium text-foreground-muted transition-colors hover:border-foreground-dimmed hover:bg-accented/40 hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-info"
      @click="openComposer"
    >
      <Icon name="i-tabler-pencil-plus" class="size-4" />
      {{ hasUpdates ? 'Write a project update' : 'Write first project update' }}
    </button>
    <form
      v-else
      class="rounded-md border border-outline bg-muted/30 p-4"
      @submit.prevent="postUpdate"
    >
      <h3 class="mb-3 text-xs font-medium text-foreground-highlighted">Project update</h3>
      <RichTextEditor
        ref="editor"
        v-model="body"
        placeholder="Share progress, decisions, or what comes next…"
        aria-label="Project update"
        class="min-h-25 text-sm"
      />
      <div class="mt-4 flex justify-end gap-2">
        <Button label="Cancel" variant="ghost" size="sm" @click="open = false" />
        <Button label="Post update" type="submit" size="sm" :disabled="saving || !body.trim()" />
      </div>
    </form>
    <p v-if="error" role="alert" class="mt-2 text-xs text-error">{{ error }}</p>
  </section>
</template>
