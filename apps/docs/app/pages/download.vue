<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { Accordion } from '../components/ui/accordion';
import { Icon } from '../components/ui/icon';

interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

interface GithubRelease {
  tag_name: string;
  html_url: string;
  published_at: string;
  assets: ReleaseAsset[];
}

interface DownloadOption {
  label: string;
  detail: string;
  url: string;
  size: string;
}

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'Download Initera',
  description: 'Download the latest Initera desktop release for macOS, Windows, or Linux.',
});

const release = ref<GithubRelease | null>(null);
const loading = ref(true);
const loadFailed = ref(false);
const releasesUrl = 'https://github.com/igris-labs/initera.app/releases';
const textLink =
  'inline-flex items-center gap-1.5 text-xs text-foreground-highlighted no-underline hover:underline underline-offset-4';
const questions = [
  {
    value: 'account',
    label: 'Do I need to create an account?',
    content:
      'No. The current desktop app works with a local workspace and does not require an account.',
  },
  {
    value: 'updates',
    label: 'How do updates work?',
    content:
      'Initera can check for an available release from the Help menu. You can also download the latest installer from this page or GitHub.',
  },
  {
    value: 'move',
    label: 'How do I move my work to another computer?',
    content:
      'Create a backup in Settings, copy that backup to the other computer, and import it there. Review the import summary before restoring.',
  },
];

const platforms = computed(() => {
  const assets =
    release.value?.assets.filter((asset) =>
      asset.browser_download_url.startsWith(`${releasesUrl}/download/`),
    ) ?? [];

  const option = (asset: ReleaseAsset, label: string, detail: string): DownloadOption => ({
    label,
    detail,
    url: asset.browser_download_url,
    size: `${(asset.size / 1024 / 1024).toFixed(1)} MB`,
  });

  return [
    {
      name: 'macOS',
      icon: 'i-tabler-brand-apple',
      options: assets
        .filter((asset) => asset.name.toLowerCase().endsWith('.dmg'))
        .map((asset) =>
          option(
            asset,
            asset.name.includes('aarch64')
              ? 'Apple silicon'
              : asset.name.includes('x64') || asset.name.includes('x86_64')
                ? 'Intel'
                : 'Download for Mac',
            'DMG installer',
          ),
        ),
    },
    {
      name: 'Windows',
      icon: 'i-tabler-brand-windows',
      options: assets
        .filter((asset) => asset.name.toLowerCase().endsWith('.exe'))
        .map((asset) => option(asset, 'Download for Windows', 'EXE installer')),
    },
    {
      name: 'Linux',
      icon: 'i-tabler-brand-ubuntu',
      options: assets
        .filter((asset) => /\.appimage$|\.deb$/i.test(asset.name))
        .map((asset) =>
          option(
            asset,
            asset.name.toLowerCase().endsWith('.deb') ? 'Debian / Ubuntu' : 'AppImage',
            asset.name.toLowerCase().endsWith('.deb') ? 'DEB package' : 'Portable image',
          ),
        ),
    },
  ];
});

onMounted(async () => {
  try {
    const response = await fetch(
      'https://api.github.com/repos/igris-labs/initera.app/releases/latest',
      {
        headers: { Accept: 'application/vnd.github+json' },
      },
    );

    if (response.status === 404) return;
    if (!response.ok) throw new Error(`GitHub returned ${response.status}`);

    const latest = (await response.json()) as GithubRelease;
    if (!latest.tag_name || !Array.isArray(latest.assets)) {
      throw new Error('GitHub returned an incomplete release');
    }

    release.value = latest;
  } catch {
    loadFailed.value = true;
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="mx-auto min-h-[65vh] w-full max-w-[82rem] px-4 py-24 sm:px-6 lg:py-36">
    <div class="max-w-[54rem]">
      <p class="text-[0.7rem] font-semibold tracking-[0.14em] text-foreground-muted uppercase">
        DESKTOP APP
      </p>
      <h1
        class="mt-7 max-w-[55rem] text-balance text-[clamp(3.2rem,13vw,5rem)] leading-none font-semibold tracking-[-0.085em] text-foreground-highlighted sm:text-[clamp(3.5rem,6.5vw,6.5rem)]"
      >
        Download Initera.
      </h1>
      <p class="mt-8 max-w-[42rem] text-[1.05rem] leading-relaxed text-foreground-muted">
        Choose the installer for your computer. Release notes and files are published on GitHub.
      </p>
      <p
        v-if="release"
        class="mt-8 flex flex-wrap items-baseline gap-1 text-[0.82rem] text-foreground-muted"
      >
        Latest release <strong>{{ release.tag_name }}</strong>
        <span v-if="release.published_at">
          ·
          {{
            new Date(release.published_at).toLocaleDateString('en', {
              day: 'numeric',
              month: 'long',
              year: 'numeric',
            })
          }}</span
        >
      </p>
      <p v-else-if="loading" class="mt-8 text-[0.82rem] text-foreground-muted" role="status">
        Checking latest release…
      </p>
      <p v-else class="mt-8 text-[0.82rem] text-foreground-muted" role="status">
        {{
          loadFailed
            ? 'Release information is temporarily unavailable.'
            : 'No public release found yet.'
        }}
        <a
          :href="releasesUrl"
          target="_blank"
          rel="noopener noreferrer"
          class="ml-1 text-foreground-highlighted underline underline-offset-4"
          >View GitHub releases</a
        >
      </p>
    </div>

    <section class="mt-20 grid gap-4 lg:grid-cols-3" aria-label="Download by platform">
      <article
        v-for="platform in platforms"
        :key="platform.name"
        class="min-h-64 rounded-[0.6rem] border border-outline bg-muted p-6 lg:min-h-68 lg:p-9"
      >
        <Icon :name="platform.icon" size="lg" aria-hidden="true" class="text-foreground-muted" />
        <h2 class="mt-5 text-xl font-semibold tracking-tight text-foreground-highlighted">
          {{ platform.name }}
        </h2>
        <div v-if="platform.options.length" class="mt-8 grid gap-2">
          <a
            v-for="download in platform.options"
            :key="download.url"
            :href="download.url"
            class="flex min-h-14 items-center justify-between gap-4 rounded-md border border-outline bg-default px-3.5 py-2.5 text-foreground-highlighted no-underline transition-[background,border-color,transform] duration-200 hover:-translate-y-0.5 hover:border-outline hover:bg-accented focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info"
          >
            <span class="grid gap-0.5">
              <strong class="text-[0.8rem] font-semibold">{{ download.label }}</strong>
              <small class="text-xs text-foreground-muted"
                >{{ download.detail }} · {{ download.size }}</small
              >
            </span>
            <Icon name="i-tabler-arrow-down" size="sm" aria-hidden="true" />
          </a>
        </div>
        <p v-else class="mt-8 text-xs leading-relaxed text-foreground-muted">
          {{
            loading
              ? 'Checking availability…'
              : release
                ? 'No installer in this release.'
                : 'Available after a public release.'
          }}
        </p>
      </article>
    </section>

    <div class="mt-10 flex flex-wrap items-center gap-x-8 gap-y-2 text-xs text-foreground-muted">
      <p>Looking for an older version or release notes?</p>
      <a :href="releasesUrl" target="_blank" rel="noopener noreferrer" :class="textLink">
        All GitHub releases <Icon name="i-tabler-arrow-up-right" size="sm" />
      </a>
      <NuxtLink to="/getting-started/quick-start" :class="textLink">
        Installation guide <Icon name="i-tabler-arrow-right" size="sm" />
      </NuxtLink>
    </div>

    <section
      class="mt-24 grid gap-12 pt-12 lg:grid-cols-[0.8fr_1.2fr] lg:gap-20"
      aria-labelledby="install-heading"
    >
      <div>
        <h2
          id="install-heading"
          class="text-3xl font-semibold tracking-tight text-foreground-highlighted"
        >
          After downloading
        </h2>
        <p class="mt-4 max-w-md text-sm leading-7 text-foreground-muted">
          Open the file for your platform and follow its install steps. Launch Initera, then create
          a workspace to begin. Your first workspace is stored on that computer.
        </p>
      </div>
      <ol class="border-t border-outline text-sm">
        <li class="grid grid-cols-[2rem_1fr] gap-4 border-b border-outline py-5">
          <span class="text-foreground-dimmed">01</span
          ><span>Install and open the desktop app.</span>
        </li>
        <li class="grid grid-cols-[2rem_1fr] gap-4 border-b border-outline py-5">
          <span class="text-foreground-dimmed">02</span
          ><span>Create a workspace and your first issue or project.</span>
        </li>
        <li class="grid grid-cols-[2rem_1fr] gap-4 border-b border-outline py-5">
          <span class="text-foreground-dimmed">03</span
          ><span>Use Settings to make a backup when you have work to keep.</span>
        </li>
      </ol>
    </section>

    <section
      class="mt-24 grid gap-12 pt-12 lg:grid-cols-[0.8fr_1.2fr] lg:gap-20"
      aria-labelledby="download-faq-heading"
    >
      <h2
        id="download-faq-heading"
        class="text-3xl font-semibold tracking-tight text-foreground-highlighted"
      >
        Questions
      </h2>
      <Accordion
        :items="questions"
        collapsible
        size="md"
        :ui="{
          root: 'border-t border-outline',
          trigger: 'py-5 text-left text-base font-medium text-foreground-highlighted',
          content: 'pb-5 text-sm leading-7 text-foreground-muted',
        }"
      />
    </section>
  </div>
</template>
