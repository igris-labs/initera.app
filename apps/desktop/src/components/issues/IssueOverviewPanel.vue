<script setup lang="ts">
import { computed, ref } from 'vue';
import { labelSwatch, priorityColor } from '../../lib/issue-presentation';
import { defaultProjectIcon, iconColor } from '../../lib/icon-presentation';
import { useProjectsStore } from '../../stores/projects';
import type { Issue, Priority } from '../../types/issue';
import { Icon } from '../ui/icon';
import { Tabs } from '../ui/tabs';

interface Props {
  items: Issue[];
}

const props = defineProps<Props>();

const projects = useProjectsStore();

function projectIcon(name: string) {
  const project = projects.items.find((item) => item.name === name);
  return {
    name: project?.icon || defaultProjectIcon,
    className: project ? iconColor(project.color).text : 'text-foreground-muted',
  };
}

const tab = ref<'projects' | 'labels' | 'priority'>('projects');

const tabs = [
  { label: 'Projects', value: 'projects' },
  { label: 'Labels', value: 'labels' },
  { label: 'Priority', value: 'priority' },
] as const;

const rows = computed(() => {
  const counts = new Map<string, number>();
  for (const issue of props.items) {
    const values =
      tab.value === 'projects'
        ? [issue.project]
        : tab.value === 'priority'
          ? [issue.priority]
          : issue.labels;
    for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
  }

  return [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
});
</script>

<template>
  <aside
    aria-label="Issue overview"
    class="workbench-surface flex h-full w-full flex-col overflow-hidden rounded-lg border border-outline bg-elevated shadow-lg"
  >
    <Tabs
      v-model="tab"
      :items="[...tabs]"
      aria-label="Overview by"
      unmount-on-hide
      size="sm"
      :ui="{
        root: 'flex min-h-0 flex-1 flex-col',
        list: 'flex gap-1 bg-transparent p-2',
        tab: 'min-w-0 flex-1 rounded-md px-2 py-1.5 text-xs font-medium text-foreground-muted transition-colors duration-150 ease-out hover:bg-muted hover:text-foreground data-[akaza-state=active]:bg-accented data-[akaza-state=active]:text-foreground-highlighted data-[akaza-state=active]:shadow-none data-[akaza-state=active]:ring-0',
        indicator: 'hidden',
        panels: 'min-h-0 flex-1 overflow-y-auto px-2 pb-2',
        panel: 'pt-0',
      }"
    >
      <template v-for="item in tabs" :key="item.value" #[`panel-${item.value}`]>
        <div v-if="!rows.length" class="px-2 py-4 text-xs text-foreground-muted">
          No {{ tab }} in this view
        </div>
        <div
          v-for="[name, count] in rows"
          :key="name"
          class="flex min-h-8 items-center gap-2 rounded-md px-2 text-xs text-foreground transition-colors duration-150 ease-out hover:bg-muted"
        >
          <Icon
            v-if="tab === 'projects'"
            :name="projectIcon(name).name"
            :class="['size-4 shrink-0', projectIcon(name).className]"
          />
          <Icon
            v-else-if="tab === 'priority'"
            name="i-tabler-antenna-bars-5"
            :class="['size-4 shrink-0', priorityColor[name as Priority]]"
          />
          <span
            v-else
            class="size-2.5 shrink-0 rounded-full"
            :style="{ backgroundColor: labelSwatch(name) }"
          />
          <span class="min-w-0 flex-1 truncate">{{ name }}</span>
          <span class="tabular-nums text-foreground-muted">{{ count }}</span>
        </div>
      </template>
    </Tabs>
  </aside>
</template>
