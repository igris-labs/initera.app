<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { Icon } from '../components/ui/icon';

interface Release {
  id: number;
  name: string;
  tag_name: string;
  body: string;
  html_url: string;
  published_at: string;
  draft: boolean;
  prerelease: boolean;
}

definePageMeta({ layout: 'marketing' });
useSeoMeta({
  title: 'Changelog | Initera',
  description: 'Read recent Initera desktop release notes and download links from GitHub.',
});

const releases = ref<Release[]>([]);
const loading = ref(true);
const failed = ref(false);
const releasesUrl = 'https://github.com/igris-labs/initera.app/releases';

function summary(body: string) {
  return (
    body
      .split('\n')
      .map((line) =>
        line
          .trim()
          .replace(/^[-*] /, '')
          .replace(/\*\*/g, '')
          .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1'),
      )
      .find((line) => line && !line.startsWith('#') && !line.startsWith('![')) ||
    'Read the release notes on GitHub.'
  );
}

onMounted(async () => {
  try {
    const response = await fetch(
      'https://api.github.com/repos/igris-labs/initera.app/releases?per_page=12',
      {
        headers: { Accept: 'application/vnd.github+json' },
      },
    );
    if (!response.ok) throw new Error(`GitHub returned ${response.status}`);
    const result = (await response.json()) as Release[];
    if (!Array.isArray(result)) throw new Error('Invalid release list');
    releases.value = result.filter((release) => !release.draft && !release.prerelease);
  } catch {
    failed.value = true;
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="mx-auto min-h-[65vh] w-full max-w-[82rem] px-4 py-20 sm:px-6 lg:py-28">
    <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">Releases</p>
    <h1
      class="mt-5 text-[clamp(2.7rem,5vw,4.5rem)] leading-tight font-semibold tracking-tight text-foreground-highlighted"
    >
      Changelog
    </h1>
    <p class="mt-5 max-w-[43rem] text-base leading-7 text-foreground-muted">
      Desktop releases and their notes are published on GitHub. This list updates when a new release
      is available.
    </p>

    <p v-if="loading" class="mt-14 text-sm text-foreground-muted" role="status">
      Loading releases…
    </p>
    <p
      v-else-if="failed || !releases.length"
      class="mt-14 text-sm leading-7 text-foreground-muted"
      role="status"
    >
      Release history is unavailable here.
      <a
        :href="releasesUrl"
        target="_blank"
        rel="noopener noreferrer"
        class="text-foreground-highlighted underline underline-offset-4"
        >View releases on GitHub</a
      >.
    </p>
    <ol v-else class="mt-14 max-w-[57rem] border-t border-outline">
      <li
        v-for="release in releases"
        :key="release.id"
        class="grid gap-3 border-b border-outline py-8 sm:grid-cols-[10rem_1fr] sm:gap-8"
      >
        <div class="text-sm text-foreground-muted">
          <time :datetime="release.published_at">{{
            new Date(release.published_at).toLocaleDateString('en', {
              day: 'numeric',
              month: 'long',
              year: 'numeric',
            })
          }}</time>
        </div>
        <div>
          <h2 class="text-xl font-semibold text-foreground-highlighted">
            {{ release.name || release.tag_name }}
          </h2>
          <p class="mt-2 text-sm leading-7 text-foreground-muted">
            {{ summary(release.body || '') }}
          </p>
          <a
            :href="release.html_url"
            target="_blank"
            rel="noopener noreferrer"
            class="mt-4 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted hover:underline underline-offset-4"
            >Full notes and downloads <Icon name="i-tabler-arrow-up-right" size="sm"
          /></a>
        </div>
      </li>
    </ol>
  </div>
</template>
