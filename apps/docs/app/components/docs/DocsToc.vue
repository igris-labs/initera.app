<script setup lang="ts">
import type { NavigationMenuItem } from 'akaza-ui';
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { Icon } from '../ui/icon';
import { NavigationMenu } from '../ui/navigation-menu';

interface TocLink {
  id: string;
  text: string;
  depth: number;
  children?: TocLink[];
}

interface Props {
  links: TocLink[];
}

const { links } = defineProps<Props>();
const route = useRoute();
const activeId = ref('');
const list = ref<HTMLElement | null>(null);
const indicatorTop = ref(0);
const indicatorHeight = ref(0);
const indicatorReady = ref(false);

const visibleLinks = computed(() => links.flatMap((link) => [link, ...(link.children ?? [])]));
const items = computed<NavigationMenuItem[]>(() =>
  visibleLinks.value.map((link, index) => ({
    value: link.id,
    label: link.text,
    href: `#${link.id}`,
    depth: link.depth,
    active: activeId.value === link.id || (!activeId.value && index === 0),
  })),
);

function updateActiveHeading() {
  const headings = visibleLinks.value
    .map((link) => document.getElementById(link.id))
    .filter((heading): heading is HTMLElement => Boolean(heading));

  if (!headings.length) return;

  // Switch while the next section is readable, before its heading reaches the header.
  const activationLine = Math.max(112, window.innerHeight * 0.4);
  let current = headings[0]!;
  for (const heading of headings) {
    if (heading.getBoundingClientRect().top <= activationLine) current = heading;
  }

  if (
    window.scrollY > 0 &&
    window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2
  )
    current = headings.at(-1)!;

  activeId.value = current.id;
}

function updateIndicator() {
  const activeLink = list.value?.querySelector<HTMLElement>('a[data-akaza-active]');
  if (!list.value || !activeLink) return;

  const listBounds = list.value.getBoundingClientRect();
  const linkBounds = activeLink.getBoundingClientRect();
  indicatorTop.value = linkBounds.top - listBounds.top;
  indicatorHeight.value = linkBounds.height;

  if (!indicatorReady.value) requestAnimationFrame(() => (indicatorReady.value = true));
}

let scheduledFrame = 0;

function scheduleUpdate() {
  if (scheduledFrame) return;
  scheduledFrame = requestAnimationFrame(async () => {
    scheduledFrame = 0;
    updateActiveHeading();
    await nextTick();
    updateIndicator();
  });
}

onMounted(() => {
  window.addEventListener('scroll', scheduleUpdate, { passive: true });
  window.addEventListener('resize', scheduleUpdate);
  scheduleUpdate();
});

onUnmounted(() => {
  window.removeEventListener('scroll', scheduleUpdate);
  window.removeEventListener('resize', scheduleUpdate);
  cancelAnimationFrame(scheduledFrame);
});

watch([visibleLinks, () => route.path], async () => {
  await nextTick();
  if (import.meta.client) scheduleUpdate();
});
</script>

<template>
  <aside
    v-if="items.length"
    aria-label="On this page"
    class="sticky top-16 max-h-[calc(100dvh-4rem)] overflow-y-auto py-1"
  >
    <h2 class="mb-3 flex items-center gap-2 text-xs font-medium text-foreground-muted">
      <Icon name="i-tabler-list-details" size="xs" />
      On this page
    </h2>

    <div ref="list" class="relative">
      <span
        v-if="indicatorHeight"
        aria-hidden="true"
        class="pointer-events-none absolute left-0 z-10 w-px bg-foreground-highlighted motion-reduce:transition-none"
        :class="indicatorReady ? 'transition-[transform,height] duration-200 ease-out' : ''"
        :style="{
          height: `${indicatorHeight}px`,
          transform: `translateY(${indicatorTop}px)`,
        }"
      />
      <NavigationMenu
        :items="items"
        orientation="vertical"
        content-mode="inline"
        aria-label="Page sections"
        :ui="{
          root: 'max-w-52',
          list: 'flex w-full flex-col items-stretch border-l border-outline',
          item: 'w-full',
          link: 'block -ml-px w-full min-h-7 rounded-none border-l border-transparent px-3 py-1 text-xs leading-5 text-foreground-dimmed outline-none transition-colors duration-200 hover:text-foreground-highlighted focus-visible:text-foreground-highlighted data-akaza-active:text-foreground-highlighted motion-reduce:transition-none',
          inlineLink: 'rounded-none',
        }"
      >
        <template #item-label="{ item }">
          <span :class="Number(item.depth) > 2 ? 'block pl-2' : 'block'">{{ item.label }}</span>
        </template>
      </NavigationMenu>
    </div>
  </aside>
</template>
