<script setup lang="ts">
import { ref, watch } from 'vue';
import type { DocsFaqItem } from '../../lib/docs-faq';
import { Accordion } from '../ui/accordion';

interface Props {
  items: DocsFaqItem[];
}

const { items } = defineProps<Props>();
const route = useRoute();
const openQuestion = ref('');

watch(
  [() => route.path, () => route.hash],
  () => {
    const target = decodeURIComponent(route.hash.slice(1));
    openQuestion.value = items.some((item) => item.id === target) ? target : '';
  },
  { immediate: true },
);
</script>

<template>
  <section aria-labelledby="faq" class="mt-10">
    <h2 id="faq" class="mb-4 text-xl font-semibold text-foreground-highlighted">FAQ</h2>

    <Accordion
      v-model="openQuestion"
      :items="items.map((item) => ({ ...item, value: item.id }))"
      collapsible
      size="md"
      :ui="{
        root: 'border-t border-outline',
        trigger:
          'rounded-none py-3 text-left text-sm text-foreground transition-colors hover:text-foreground-highlighted',
        content: 'pb-4 text-sm leading-6 text-foreground-muted',
      }"
    >
      <template #trigger="{ item }">
        <span :id="item.value" role="heading" aria-level="3" class="scroll-mt-20">{{
          item.label
        }}</span>
      </template>
      <template #content="{ item }">
        <ContentRenderer
          :value="{ body: { type: 'minimark', value: item.answer } }"
          class="docs-faq-answer [&>:last-child]:mb-0"
        />
      </template>
    </Accordion>
  </section>
</template>
