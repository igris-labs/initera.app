<script setup lang="ts">
import { useEventListener } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { queryIssueIdPage } from '../../lib/issue-storage';
import { notifications } from '../../lib/notifications';
import { savedViewQuery } from '../../lib/saved-view-query';
import { unavailableLabelFilters } from '../../lib/label-filter-availability';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import { useLabelsStore } from '../../stores/labels';
import { useViewsStore } from '../../stores/views';
import { Button } from '../../components/ui/button';
import { Icon } from '../../components/ui/icon';
import { Input } from '../../components/ui/input';

const views = useViewsStore();
const issues = useIssuesStore();
const labels = useLabelsStore();
const router = useRouter();

const creating = ref(false);
const name = ref('');
const viewCounts = ref<Record<string, number>>({});
let countRevision = 0;
const unavailableViews = computed(
  () =>
    new Set(
      labels.ready
        ? views.items
            .filter((view) => unavailableLabelFilters(savedViewQuery(view), labels.items).length)
            .map((view) => view.id)
        : [],
    ),
);

watch(
  [() => views.items, () => issues.items, () => issues.ready, unavailableViews],
  async () => {
    if (!usesNativeStorage || !issues.ready) return;

    const revision = ++countRevision;
    await issues.flushSaves();

    const results = await Promise.allSettled(
      views.items.map(async (view) => ({
        id: view.id,
        count: unavailableViews.value.has(view.id)
          ? null
          : (await queryIssueIdPage(savedViewQuery(view), 1)).totalCount,
      })),
    );

    if (revision !== countRevision) return;

    viewCounts.value = Object.fromEntries(
      results.flatMap((result) =>
        result.status === 'fulfilled' && result.value.count !== null
          ? [[result.value.id, result.value.count]]
          : [],
      ),
    );
  },
  { deep: true, immediate: true },
);

useEventListener(window, 'create-view-command', () => {
  creating.value = true;
});

async function createView() {
  if (!name.value.trim()) return;

  try {
    const view = await views.create(name.value);
    name.value = '';
    creating.value = false;
    await router.push({ name: 'saved-view', params: { viewId: view.id } });
  } catch (error) {
    notifications.add({
      title: 'Could not create view',
      description: String(error),
      type: 'error',
    });
  }
}
</script>

<template>
  <main class="min-h-0 flex-1 overflow-auto bg-default">
    <div>
      <div class="flex min-h-11 items-center justify-between gap-4 px-4 py-1.5">
        <div>
          <h2 class="text-xs font-medium text-foreground-muted">Issue views</h2>
        </div>
        <Button label="New view" icon="i-tabler-plus" variant="soft" @click="creating = true" />
      </div>
      <form
        v-if="creating"
        class="m-4 flex max-w-md items-center gap-2 rounded-lg border border-outline bg-elevated p-3"
        @submit.prevent="createView"
      >
        <Input
          v-model="name"
          autofocus
          placeholder="View name"
          aria-label="View name"
          class="min-w-0 flex-1"
        />
        <Button label="Create" type="submit" :disabled="!name.trim()" />
        <Button icon="i-tabler-x" variant="ghost" aria-label="Cancel" @click="creating = false" />
      </form>
      <div class="overflow-hidden bg-default px-2">
        <RouterLink
          v-for="view in views.items"
          :key="view.id"
          :to="{ name: 'saved-view', params: { viewId: view.id } }"
          class="flex min-h-10 items-center gap-3 rounded-md px-3 text-(length:--ui-sidebar-text) text-foreground-highlighted transition-colors duration-150 ease-out hover:bg-muted focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-info"
        >
          <Icon name="i-tabler-stack-2" class="size-4 text-foreground-muted" />
          <span class="min-w-0 flex-1 truncate">{{ view.name }}</span>
          <span class="text-xs text-foreground-muted">{{
            view.layout === 'board' ? 'Board' : 'List'
          }}</span>
          <span v-if="unavailableViews.has(view.id)" class="text-xs text-warning"
            >Unavailable label</span
          >
          <span
            v-else-if="viewCounts[view.id] !== undefined"
            :aria-label="`${viewCounts[view.id]} matching issues`"
            class="min-w-5 text-right text-xs tabular-nums text-foreground-muted"
            >{{ viewCounts[view.id] }}</span
          >
        </RouterLink>
      </div>
    </div>
  </main>
</template>
