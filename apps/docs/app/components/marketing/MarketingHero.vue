<script setup lang="ts">
import { ref } from 'vue';
import { buttonVariants } from '../ui/button';
import { Icon } from '../ui/icon';
import { Tabs } from '../ui/tabs';

const activePreview = ref('issues');
const previews = [
  {
    value: 'issues',
    label: 'Issues',
    image: '/images/app/issues-list.png',
    alt: 'Issue list grouped by status with project and priority details',
  },
  {
    value: 'projects',
    label: 'Projects',
    image: '/images/app/project-overview.png',
    alt: 'Project overview showing milestones, updates, and issues',
  },
  {
    value: 'cycles',
    label: 'Cycles',
    image: '/images/app/cycles.png',
    alt: 'Cycles page showing current and upcoming work periods',
  },
];
</script>

<template>
  <section class="relative" aria-labelledby="hero-title">
    <div class="relative mx-auto w-full max-w-[82rem] px-4 pt-20 pb-16 sm:px-6 sm:pt-28 lg:pt-36">
      <div class="mx-auto max-w-[70rem] text-center">
        <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
          Open source · Local first
        </p>
        <h1
          id="hero-title"
          class="mt-6 text-balance text-[clamp(2.35rem,5vw,4.6rem)] leading-[1.08] font-semibold tracking-[-0.07em] text-foreground-highlighted"
        >
          See the whole project.<br class="hidden sm:block" />
          Work on the next issue.
        </h1>
        <p class="mx-auto mt-6 max-w-[40rem] text-base leading-7 text-foreground-muted sm:text-lg">
          Bring projects, issues, and cycles into one desktop workspace. Keep the plan close to the
          work, with your data on your computer.
        </p>
        <div class="mt-8 flex flex-wrap items-center justify-center gap-3">
          <NuxtLink
            to="/download"
            :class="[
              buttonVariants({ variant: 'solid', size: 'lg' }).root(),
              'min-h-11 gap-2 px-5 no-underline',
            ]"
            >Download Initera <Icon name="i-tabler-arrow-down" size="sm"
          /></NuxtLink>
          <NuxtLink
            to="/product"
            :class="[
              buttonVariants({ variant: 'outline', size: 'lg' }).root(),
              'min-h-11 px-5 no-underline',
            ]"
            >Explore the product</NuxtLink
          >
        </div>
      </div>

      <div class="relative mx-auto mt-16 max-w-[73rem] sm:mt-20">
        <Tabs
          v-model="activePreview"
          :items="previews"
          aria-label="Preview Initera"
          size="md"
          variant="soft"
          :ui="{
            root: 'relative gap-4',
            list: 'mx-auto rounded-lg border border-outline bg-muted',
            panels: 'min-w-0',
            panel: 'min-w-0',
          }"
        >
          <template v-for="preview in previews" :key="preview.value" #[`panel-${preview.value}`]>
            <figure
              class="overflow-hidden rounded-xl border border-outline bg-muted shadow-[0_3rem_7rem_-3rem_#000]"
            >
              <NuxtImg
                :src="preview.image"
                :alt="preview.alt"
                width="1440"
                height="900"
                sizes="100vw lg:1168px"
                :loading="preview.value === 'issues' ? 'eager' : 'lazy'"
                class="block aspect-[1.35] w-full object-cover object-left-top sm:aspect-auto sm:h-auto"
              />
            </figure>
          </template>
        </Tabs>
        <p class="mt-4 text-center text-xs text-foreground-dimmed">A look inside the desktop app</p>
      </div>
    </div>
  </section>
</template>
