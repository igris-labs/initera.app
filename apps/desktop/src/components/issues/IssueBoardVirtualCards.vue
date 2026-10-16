<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual';
import { computed, ref } from 'vue';
import type { ComponentPublicInstance } from 'vue';
import type { Issue } from '../../types/issue';
import IssueBoardCardContent from './IssueBoardCardContent.vue';

interface Props {
  items: Issue[];
  lane: string;
  showPriority: boolean;
  showProject: boolean;
  columns?: string[];
  density?: 'compact' | 'default';
  openIssue: (id: string) => void;
  moveWithKeyboard: (event: KeyboardEvent, issue: Issue) => void;
}

const props = defineProps<Props>();
const selectedId = defineModel<string | null>('selectedId', { required: true });
const scrollElement = ref<HTMLElement | null>(null);

const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.items.length,
    getScrollElement: () => scrollElement.value,
    estimateSize: () => (props.density === 'compact' ? 92 : 112),
    getItemKey: (index: number) => props.items[index]?.id ?? index,
    overscan: 5,
  })),
);

const visible = computed(() => virtualizer.value.getVirtualItems());
const top = computed(() => visible.value[0]?.start ?? 0);
const bottom = computed(() => virtualizer.value.getTotalSize() - (visible.value.at(-1)?.end ?? 0));

function measureCard(element: Element | ComponentPublicInstance | null) {
  if (element instanceof HTMLElement) virtualizer.value.measureElement(element);
}
</script>

<template>
  <div ref="scrollElement" class="max-h-[calc(100vh-13rem)] min-h-28 overflow-y-auto">
    <div :data-board-lane="lane" :data-board-offset="visible[0]?.index ?? 0">
      <div :style="{ height: `${top}px` }" aria-hidden="true" />
      <div
        v-for="item in visible"
        :key="String(item.key)"
        :ref="measureCard"
        :data-index="item.index"
        :data-issue-id="items[item.index]?.id"
        role="button"
        tabindex="0"
        :aria-label="items[item.index]?.title"
        aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight"
        class="group cursor-default rounded-lg pb-2 focus-visible:outline-2 focus-visible:outline-info"
        @click="items[item.index] && openIssue(items[item.index]!.id)"
        @keydown="items[item.index] && moveWithKeyboard($event, items[item.index]!)"
        @keydown.enter="selectedId = items[item.index]?.id ?? null"
        @keydown.space.prevent="selectedId = items[item.index]?.id ?? null"
      >
        <div
          :class="[
            'rounded-lg border border-outline bg-elevated transition-[border-color,background-color] duration-150 ease-out group-hover:border-outline/80 group-hover:bg-accented/50',
            density === 'compact' ? 'p-2' : 'p-3',
          ]"
        >
          <IssueBoardCardContent
            v-if="items[item.index]"
            :issue="items[item.index]!"
            :show-priority="showPriority"
            :show-project="showProject"
            :columns="columns"
            :density="density"
          />
        </div>
      </div>
      <div :style="{ height: `${bottom}px` }" aria-hidden="true" />
    </div>
  </div>
</template>
