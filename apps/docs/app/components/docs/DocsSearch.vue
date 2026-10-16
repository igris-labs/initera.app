<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Listbox } from '../ui/listbox';

interface SearchResult {
  id: string;
  title: string;
  detail: string;
  section: string;
  icon: string;
}

const open = defineModel<boolean>('open', { required: true });
const query = ref('');
const activeIndex = ref(0);
const input = ref<{ inputRef?: HTMLInputElement } | null>(null);
const { data: docsNavigation } = await useDocsNavigation();

const { data: sections } = await useAsyncData('docs-search-sections', () =>
  queryCollectionSearchSections('docs'),
);

const pages = computed(() =>
  (docsNavigation.value ?? []).flatMap((group) =>
    group.items.map((page) => ({
      id: page.path,
      title: page.title,
      detail: page.description,
      section: group.title,
      icon: page.icon,
    })),
  ),
);

const results = computed<SearchResult[]>(() => {
  const term = query.value.trim().toLocaleLowerCase();

  if (!term) return pages.value;

  const matchingPages = pages.value.filter((page) =>
    `${page.title} ${page.detail}`.toLocaleLowerCase().includes(term),
  );
  const matchingSections = (sections.value ?? [])
    .filter((section) => `${section.title} ${section.content}`.toLocaleLowerCase().includes(term))
    .flatMap((section) => {
      const parent = pages.value.find(
        (page) => section.id === page.id || section.id.startsWith(`${page.id}#`),
      );
      if (!parent) return [];

      return [
        {
          id: section.id,
          title: section.title,
          detail: parent.title,
          section: parent.section,
          icon: parent.icon,
        },
      ];
    });

  const seen = new Set<string>();
  const groupOrder = new Map(
    (docsNavigation.value ?? []).map((group, index) => [group.title, index]),
  );
  const pageOrder = new Map(pages.value.map((page, index) => [page.id, index]));

  return [...matchingPages, ...matchingSections]
    .filter((result) => {
      if (seen.has(result.id)) return false;
      seen.add(result.id);
      return true;
    })
    .sort(
      (a, b) =>
        (groupOrder.get(a.section) ?? Infinity) - (groupOrder.get(b.section) ?? Infinity) ||
        (pageOrder.get(a.id.split('#')[0] ?? '') ?? Infinity) -
          (pageOrder.get(b.id.split('#')[0] ?? '') ?? Infinity) ||
        Number(a.id.includes('#')) - Number(b.id.includes('#')),
    )
    .slice(0, 12);
});

const options = computed(() => {
  return results.value.flatMap((result, index) => {
    const label =
      results.value[index - 1]?.section === result.section
        ? []
        : [{ type: 'label' as const, label: result.section }];

    return [...label, { value: result.id, label: result.title }];
  });
});

function resultFor(value: unknown) {
  return results.value.find((result) => result.id === value);
}

function highlightResult(option?: { value?: unknown }) {
  const index = results.value.findIndex((result) => result.id === option?.value);
  if (index >= 0) activeIndex.value = index;
}

async function choose(value: unknown) {
  const result = resultFor(value);
  if (!result) return;

  input.value?.inputRef?.blur();
  open.value = false;
  await nextTick();
  await navigateTo(result.id);
}

function onKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    open.value = false;
  } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    activeIndex.value =
      (activeIndex.value + (event.key === 'ArrowDown' ? 1 : -1) + results.value.length) %
      (results.value.length || 1);
  } else if (event.key === 'Enter' && results.value[activeIndex.value]) {
    event.preventDefault();
    void choose(results.value[activeIndex.value]?.id);
  }
}

watch(query, () => {
  activeIndex.value = 0;
});

watch(open, async (isOpen) => {
  if (!isOpen) {
    query.value = '';
    return;
  }

  await nextTick();
  requestAnimationFrame(() => {
    if (open.value) input.value?.inputRef?.focus();
  });
});
</script>

<template>
  <Dialog
    v-model:open="open"
    title="Search documentation"
    size="lg"
    class="command-bar-dialog workbench-surface max-w-2xl"
    :ui="{ header: 'sr-only', body: 'p-0', footer: 'hidden' }"
  >
    <template #body>
      <Input
        ref="input"
        v-model="query"
        icon="i-tabler-search"
        aria-label="Search documentation"
        placeholder="Type a page or search..."
        class="w-full px-2 py-2"
        :ui="{ root: 'border-0 bg-transparent shadow-none focus:ring-0' }"
        role="combobox"
        aria-autocomplete="list"
        aria-controls="docs-search-results-content"
        :aria-activedescendant="
          results[activeIndex]
            ? `docs-search-results-content-option-${options.findIndex((option) => 'value' in option && option.value === results[activeIndex]?.id)}`
            : undefined
        "
        @keydown="onKeydown"
      />

      <Listbox
        id="docs-search-results"
        aria-label="Documentation pages"
        :options="options"
        :model-value="results[activeIndex]?.id ?? ''"
        :ui="{
          root: 'w-full',
          content:
            'max-h-[min(26rem,55vh)] min-w-0 overflow-y-auto rounded-none border-x-0 border-b-0 border-t border-outline bg-transparent p-1.5 shadow-none',
          option:
            'min-h-8 px-2 text-xs data-akaza-highlighted:bg-accented data-[state=checked]:bg-accented',
          groupLabel: 'px-2 pt-2 pb-1 text-[11px] font-medium text-foreground-dimmed',
          empty: 'px-3 py-6 text-center text-xs text-foreground-muted',
        }"
        @value-change="choose"
        @highlight-change="highlightResult"
      >
        <template #option="{ value }">
          <Icon
            :name="resultFor(value)?.icon ?? 'i-tabler-hash'"
            class="mr-2.5 size-4 shrink-0 text-foreground-muted"
          />
          <span class="min-w-0 flex-1 truncate">{{ resultFor(value)?.title }}</span>
          <span
            v-if="query && resultFor(value)?.detail"
            class="ml-auto max-w-32 truncate text-foreground-dimmed"
          >
            {{ resultFor(value)?.detail }}
          </span>
        </template>
        <template #empty>No matching pages or sections</template>
      </Listbox>

      <div
        class="flex items-center justify-between border-t border-outline px-3 py-2 text-xs text-foreground-muted"
      >
        <span>↑↓ Navigate <span class="mx-2">·</span> ↵ Open</span>
        <span class="flex items-center gap-1.5"><kbd class="font-medium">Esc</kbd> Close</span>
      </div>
    </template>
  </Dialog>
</template>
