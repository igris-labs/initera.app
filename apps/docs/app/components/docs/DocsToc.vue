<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue';
import { Icon } from '../ui/icon';

interface TocLink {
  id: string;
  text: string;
  depth: number;
  children?: TocLink[];
}

interface Props {
  links: TocLink[];
}

const props = defineProps<Props>();
const links = computed(() => props.links.flatMap((link) => [link, ...(link.children ?? [])]));
const active = ref('');
const visible = ref<string[]>([]);
const navigation = useTemplateRef<HTMLElement>('navigation');
const indicator = ref({ top: 0, height: 0 });

function measureIndicator() {
  const rows = navigation.value?.querySelectorAll<HTMLElement>('[data-visible]');
  const first = rows?.[0];
  const last = rows?.[rows.length - 1];

  indicator.value = {
    top: first?.offsetTop ?? 0,
    height: first && last ? last.offsetTop + last.offsetHeight - first.offsetTop : 0,
  };
}

useResizeObserver(navigation, measureIndicator);
watch(visible, measureIndicator, { flush: 'post' });

let frame = 0;

function update() {
  frame = 0;

  const headings = links.value
    .map((link) => document.getElementById(link.id))
    .filter((heading): heading is HTMLElement => Boolean(heading));

  let current = headings[0];
  for (const heading of headings) {
    if (heading.getBoundingClientRect().top <= 112) current = heading;
  }

  if (
    window.scrollY > 0 &&
    window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2
  )
    current = headings.at(-1);

  active.value = current?.id ?? '';

  const inViewport = headings.filter((heading) => {
    const rect = heading.getBoundingClientRect();
    return rect.bottom > 64 && rect.top < window.innerHeight;
  });

  visible.value = inViewport.length
    ? inViewport.map((heading) => heading.id)
    : current
      ? [current.id]
      : [];
}

function schedule() {
  if (!frame) frame = requestAnimationFrame(update);
}

onMounted(() => {
  window.addEventListener('scroll', schedule, { passive: true });
  window.addEventListener('resize', schedule);
  schedule();
});

onUnmounted(() => {
  window.removeEventListener('scroll', schedule);
  window.removeEventListener('resize', schedule);
  cancelAnimationFrame(frame);
});

watch(
  links,
  () => {
    if (import.meta.client) {
      schedule();
      measureIndicator();
    }
  },
  { flush: 'post' },
);
</script>

<template>
  <aside
    v-if="links.length"
    aria-label="On this page"
    class="sticky top-16 max-h-[calc(100dvh-4rem)] overflow-y-auto py-1"
  >
    <h2 class="mb-3 flex items-center gap-2 text-xs font-medium text-foreground-muted">
      <Icon name="i-tabler-list-details" size="xs" />
      On this page
    </h2>

    <nav
      ref="navigation"
      aria-label="Page sections"
      class="relative flex max-w-52 flex-col border-l border-outline"
    >
      <span
        aria-hidden="true"
        data-testid="toc-indicator"
        class="pointer-events-none absolute top-0 -left-px w-px bg-foreground-highlighted transition-[transform,height,opacity] duration-180 ease-interaction motion-reduce:transition-none"
        :style="{
          transform: `translateY(${indicator.top}px)`,
          height: `${indicator.height}px`,
          opacity: indicator.height ? 1 : 0,
        }"
      />
      <a
        v-for="link in links"
        :key="link.id"
        :href="`#${link.id}`"
        :aria-current="active === link.id ? 'location' : undefined"
        :data-visible="visible.includes(link.id) || undefined"
        class="min-h-7 px-3 py-1 text-xs leading-5 transition-colors duration-150 ease-interaction outline-none hover:text-foreground-highlighted focus-visible:outline-1 focus-visible:outline-ring motion-reduce:transition-none"
        :class="[
          visible.includes(link.id) ? 'text-foreground-highlighted' : 'text-foreground-dimmed',
          link.depth > 2 ? 'pl-5' : '',
        ]"
        >{{ link.text }}</a
      >
    </nav>
  </aside>
</template>
