<script setup lang="ts">
import { computed, ref } from 'vue';
import DocsFaq from '../components/docs/DocsFaq.vue';
import DocsToc from '../components/docs/DocsToc.vue';
import { Button } from '../components/ui/button';
import { Icon } from '../components/ui/icon';
import { extractFaq } from '../lib/docs-faq';

const route = useRoute();
const copied = ref(false);
const { data: docsNavigation } = await useDocsNavigation();

const { data: page } = await useAsyncData(
  () => `docs-page:${route.path}`,
  () => queryCollection('docs').path(route.path).first(),
);

if (!page.value) {
  throw createError({ statusCode: 404, statusMessage: 'Page not found' });
}

const faq = computed(() => extractFaq(page.value?.body.value));
const articlePage = computed(() => {
  if (!page.value || !faq.value) return page.value;

  const { start, end } = faq.value;
  return {
    ...page.value,
    body: {
      ...page.value.body,
      value: [...page.value.body.value.slice(0, start), ...page.value.body.value.slice(end)],
    },
  };
});

const adjacentPages = computed(() => {
  const pages = docsNavigation.value?.flatMap((group) => group.items) ?? [];
  const index = pages.findIndex((item) => item.path === route.path);

  return {
    previous: index > 0 ? pages[index - 1] : null,
    next: index >= 0 ? (pages[index + 1] ?? null) : null,
  };
});

useSeoMeta({
  title: () => page.value?.title ?? 'Documentation',
  description: () => page.value?.description ?? 'Initera user guide',
});

async function copyLink() {
  try {
    await navigator.clipboard.writeText(window.location.href);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    copied.value = false;
  }
}
</script>

<template>
  <div
    class="grid min-h-[calc(100dvh-3rem)] grid-cols-1 xl:-ml-60 xl:w-screen xl:grid-cols-[minmax(0,1fr)_minmax(0,42rem)_minmax(0,1fr)]"
  >
    <div class="hidden xl:block" aria-hidden="true" />

    <article class="min-w-0 px-5 pb-24 pt-12 sm:px-8 xl:px-0">
      <div class="mx-auto max-w-168">
        <div class="mb-8">
          <h1
            class="text-[1.75rem] font-semibold leading-tight tracking-tight text-foreground-highlighted"
          >
            {{ page?.title }}
          </h1>
          <p class="mt-4 text-sm leading-6 text-foreground-muted">
            {{ page?.description }}
          </p>

          <div class="mt-5 flex items-center gap-2">
            <Button
              :icon="copied ? 'i-tabler-check' : 'i-tabler-link'"
              :label="copied ? 'Copied link' : 'Copy link'"
              variant="ghost"
              size="xs"
              @click="copyLink"
            />
          </div>
        </div>

        <ContentRenderer v-if="articlePage" :value="articlePage" class="docs-prose" />
        <DocsFaq v-if="faq" :items="faq.items" class="docs-prose" />

        <nav
          class="mt-16 grid gap-3 border-t border-outline pt-6 sm:grid-cols-2"
          aria-label="Guide pages"
        >
          <NuxtLink
            v-if="adjacentPages.previous"
            :to="adjacentPages.previous.path"
            class="group flex items-center gap-3 rounded-md border border-outline px-4 py-3 text-sm hover:bg-accented"
          >
            <Icon
              name="i-tabler-chevron-left"
              size="sm"
              class="shrink-0 text-foreground-muted transition-transform group-hover:-translate-x-0.5"
            />
            <span class="min-w-0">
              <span class="block text-xs text-foreground-muted">Previous</span>
              <span class="mt-1 block font-medium text-foreground-highlighted">{{
                adjacentPages.previous.title
              }}</span>
            </span>
          </NuxtLink>
          <NuxtLink
            v-if="adjacentPages.next"
            :to="adjacentPages.next.path"
            class="group flex items-center justify-end gap-3 rounded-md border border-outline px-4 py-3 text-right text-sm hover:bg-accented sm:col-start-2"
          >
            <span class="min-w-0">
              <span class="block text-xs text-foreground-muted">Next</span>
              <span class="mt-1 block font-medium text-foreground-highlighted">{{
                adjacentPages.next.title
              }}</span>
            </span>
            <Icon
              name="i-tabler-chevron-right"
              size="sm"
              class="shrink-0 text-foreground-muted transition-transform group-hover:translate-x-0.5"
            />
          </NuxtLink>
        </nav>
      </div>
    </article>

    <div class="hidden min-w-0 pt-12 xl:block xl:pl-24">
      <DocsToc :links="page?.body.toc?.links ?? []" />
    </div>
  </div>
</template>
