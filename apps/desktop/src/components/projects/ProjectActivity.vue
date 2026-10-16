<script setup lang="ts">
import { computed, watch } from 'vue';
import { RichTextEditor } from '../editor';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { useProjectUpdatesStore } from '../../stores/project-updates';
import { useWorkspaceStore } from '../../stores/workspace';

interface Props {
  projectId: string;
  createdAt: string;
}

const props = defineProps<Props>();

const updates = useProjectUpdatesStore();
const workspace = useWorkspaceStore();

const entries = computed(() => updates.byProject[props.projectId] ?? []);

watch(
  () => props.projectId,
  (id) => {
    void updates.load(id).catch(() => {});
  },
  { immediate: true },
);

function formatDate(value: string) {
  if (!value) return '';
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(value));
}

function initials(name: string) {
  return name
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('');
}

async function loadOlderActivity() {
  await updates.loadMore(props.projectId).catch(() => {});
}
</script>

<template>
  <section aria-label="Project activity" class="border-t border-outline pt-5">
    <h3 class="mb-4 text-xs font-medium text-foreground-muted">Activity</h3>
    <p v-if="updates.errors[projectId]" role="alert" class="mb-3 text-xs text-error">
      {{ updates.errors[projectId] }}
    </p>
    <p
      v-if="updates.loading[projectId] && !entries.length"
      class="mb-3 text-xs text-foreground-muted"
    >
      Loading activity…
    </p>
    <ol class="space-y-5">
      <li v-for="entry in entries" :key="entry.id" class="flex gap-3">
        <span
          class="flex size-7 shrink-0 items-center justify-center rounded-full bg-accented text-[10px] font-medium text-foreground-highlighted"
          >{{ initials(entry.actorName) }}</span
        >
        <div class="min-w-0 flex-1">
          <p class="text-xs leading-5">
            <span class="font-medium text-foreground-highlighted">{{ entry.actorName }}</span
            >{{ ' ' }}
            <span class="text-foreground-muted"
              >{{ entry.kind === 'change' ? entry.body : 'posted an update' }} ·
              {{ formatDate(entry.createdAt) }}</span
            >
          </p>
          <RichTextEditor
            v-if="entry.kind !== 'change'"
            :model-value="entry.body"
            readonly
            aria-label="Project update content"
            class="mt-2 text-sm"
          />
        </div>
      </li>
      <li
        v-if="!updates.loading[projectId] && !updates.hasMore[projectId]"
        class="flex items-center gap-3 text-xs text-foreground-muted"
      >
        <span class="flex size-7 shrink-0 items-center justify-center rounded-full bg-accented/70"
          ><Icon name="i-tabler-cube" class="size-3.5"
        /></span>
        <span
          >{{ workspace.userName || 'You' }} created the project<span v-if="createdAt">
            · {{ formatDate(createdAt) }}</span
          ></span
        >
      </li>
    </ol>
    <Button
      v-if="updates.hasMore[projectId]"
      label="Load older activity"
      variant="ghost"
      size="xs"
      class="mt-3"
      :disabled="updates.loading[projectId]"
      @click="loadOlderActivity"
    />
  </section>
</template>
