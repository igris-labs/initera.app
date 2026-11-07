<script setup lang="ts">
import MarketingHero from '../components/marketing/MarketingHero.vue';
import MarketingWorkflow from '../components/marketing/MarketingWorkflow.vue';
import MarketingFaq from '../components/marketing/MarketingFaq.vue';
import { buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'Initera | Projects and issues, together',
  description:
    'Plan projects, track issues, and work through cycles in an open-source desktop app. Keep your data on your computer.',
});

const shell = 'mx-auto w-full max-w-[82rem] px-4 sm:px-6';
const link =
  'inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted no-underline hover:underline underline-offset-4';
const frame =
  'overflow-hidden rounded-xl border border-outline bg-muted shadow-[0_2rem_5rem_-3rem_#000]';
</script>

<template>
  <div class="overflow-hidden bg-default">
    <MarketingHero />
    <MarketingWorkflow id="features" />

    <section id="workflow" class="scroll-mt-24 bg-muted dark:bg-[#141414]">
      <div
        :class="[
          shell,
          'grid items-center gap-12 py-20 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20 lg:py-28',
        ]"
      >
        <div>
          <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
            Find the work
          </p>
          <h2
            class="mt-5 max-w-md text-balance text-3xl leading-tight font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
          >
            Change views without changing context.
          </h2>
          <p class="mt-5 max-w-md text-base leading-7 text-foreground-muted">
            See issues as a list or board. Save a filtered view for work you return to often. Search
            by title or identifier, then use the command bar to open a result or run an action.
          </p>
          <div class="mt-7 flex flex-wrap gap-x-7 gap-y-3">
            <NuxtLink to="/planning/views-and-search" :class="link"
              >Views and search <Icon name="i-tabler-arrow-right" size="sm"
            /></NuxtLink>
            <NuxtLink to="/reference/command-bar" :class="link"
              >Command bar <Icon name="i-tabler-arrow-right" size="sm"
            /></NuxtLink>
          </div>
        </div>
        <div class="relative pb-9 sm:pr-14">
          <figure :class="frame">
            <NuxtImg
              src="/images/app/board-view.png"
              alt="Issue board with cards grouped by status in Initera"
              width="1440"
              height="900"
              sizes="100vw lg:760px"
              loading="lazy"
              class="block h-auto w-full"
            />
          </figure>
          <figure
            class="absolute right-0 bottom-0 w-[min(68%,29rem)] overflow-hidden rounded-lg border border-outline bg-default shadow-2xl"
          >
            <NuxtImg
              src="/images/app/command-bar.png"
              alt="Command bar with issue search and actions"
              width="672"
              height="496"
              sizes="68vw lg:460px"
              loading="lazy"
              class="block h-auto w-full"
            />
          </figure>
        </div>
      </div>
    </section>

    <section aria-labelledby="local-title">
      <div
        :class="[
          shell,
          'grid items-center gap-12 py-20 lg:grid-cols-[1.2fr_0.8fr] lg:gap-20 lg:py-28',
        ]"
      >
        <figure :class="[frame, 'order-2 lg:order-1']">
          <NuxtImg
            src="/images/app/settings.png"
            alt="Workspace settings and local data folder in Initera"
            width="1440"
            height="900"
            sizes="100vw lg:720px"
            loading="lazy"
            class="block h-auto w-full"
          />
        </figure>
        <div class="order-1 lg:order-2">
          <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
            Keep your copy
          </p>
          <h2
            id="local-title"
            class="mt-5 max-w-md text-balance text-3xl leading-tight font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
          >
            Your work has a place on your computer.
          </h2>
          <p class="mt-5 max-w-md text-base leading-7 text-foreground-muted">
            Initera stores workspace records in a local database and images in a media folder. You
            can make a backup that includes both, then restore it when you need to move or recover
            your work. No account is required to start.
          </p>
          <NuxtLink to="/local-first" :class="[link, 'mt-7']"
            >How local data works <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
        </div>
      </div>
    </section>

    <section
      :class="[shell, 'grid gap-12 py-20 lg:grid-cols-[0.8fr_1.2fr] lg:gap-20 lg:py-28']"
      aria-labelledby="source-title"
    >
      <div>
        <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
          Open source
        </p>
        <h2
          id="source-title"
          class="mt-5 max-w-md text-balance text-3xl leading-tight font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
        >
          See the code. Shape what comes next.
        </h2>
      </div>
      <div class="pt-6">
        <p class="max-w-2xl text-base leading-7 text-foreground-muted">
          Initera is published under the MIT License. Read the source, report a problem, or
          contribute an improvement on GitHub. The desktop app works on its own today; accounts,
          teams, and sync are planned work.
        </p>
        <div class="mt-7 flex flex-wrap gap-x-7 gap-y-3">
          <NuxtLink to="/open-source" :class="link"
            >Open-source project <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
          <NuxtLink to="/changelog" :class="link"
            >Release history <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
        </div>
      </div>
    </section>

    <MarketingFaq />

    <section
      :class="[
        shell,
        'flex flex-col items-start justify-between gap-8 py-16 sm:flex-row sm:items-center lg:py-24',
      ]"
    >
      <div>
        <h2
          class="text-balance text-3xl font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
        >
          Start with one issue.
        </h2>
        <p class="mt-3 text-base text-foreground-muted">
          Download Initera and make a workspace when you are ready.
        </p>
      </div>
      <NuxtLink
        to="/download"
        :class="[
          buttonVariants({ variant: 'solid', size: 'lg' }).root(),
          'min-h-11 shrink-0 gap-2 no-underline',
        ]"
        >Download Initera <Icon name="i-tabler-arrow-down" size="sm"
      /></NuxtLink>
    </section>
  </div>
</template>
