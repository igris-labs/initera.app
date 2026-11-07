<script setup lang="ts">
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });
useSeoMeta({
  title: 'Features | Initera',
  description:
    'Explore issue tracking, project planning, cycles, saved views, and time tracking in Initera.',
});

const features = [
  {
    title: 'Issues that keep their context',
    description:
      'Set status, priority, labels, and an owner. Add a description, comments, sub-issues, and relationships so the next person can see what needs to happen.',
    image: '/images/app/issue-detail.png',
    alt: 'Issue detail showing properties and activity',
    link: '/issues/issues',
    label: 'Explore issues',
  },
  {
    title: 'A project page for the work behind the plan',
    description:
      'Bring milestones, resources, updates, and issues into one project. Use the structure view when a project needs more than a flat list.',
    image: '/images/app/project-overview.png',
    alt: 'Project overview showing milestones and issues',
    link: '/projects/projects',
    label: 'Explore projects',
  },
  {
    title: 'Cycles with a clear finish line',
    description:
      'Choose the schedule, add work to a cycle, and review what is unfinished at the end. Upcoming cycles are prepared from the series you set.',
    image: '/images/app/cycles.png',
    alt: 'Cycle schedule in the desktop app',
    link: '/planning/cycles',
    label: 'Explore cycles',
  },
  {
    title: 'A view for the question you are asking',
    description:
      'Switch between list and board, filter the issue set, and save views you use often. The command bar finds issues and projects and runs actions from the keyboard.',
    image: '/images/app/board-view.png',
    alt: 'Issue board with work grouped by status',
    link: '/planning/views-and-search',
    label: 'Explore views',
  },
];
</script>

<template>
  <div class="bg-default">
    <header class="mx-auto w-full max-w-[82rem] px-4 pt-20 pb-16 sm:px-6 lg:pt-28">
      <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">Features</p>
      <h1
        class="mt-5 max-w-[52rem] text-balance text-[clamp(2.7rem,5vw,4.5rem)] leading-tight font-semibold tracking-tight text-foreground-highlighted"
      >
        The plan and the work, in the same place.
      </h1>
      <p class="mt-6 max-w-[43rem] text-base leading-7 text-foreground-muted">
        Initera brings issues, projects, cycles, views, and time records into a single desktop
        workspace. Use the pieces you need and add structure as the work grows.
      </p>
    </header>

    <section v-for="(feature, index) in features" :key="feature.title">
      <div
        class="mx-auto grid w-full max-w-[82rem] items-center gap-12 px-4 py-16 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:py-24"
      >
        <div :class="index % 2 ? 'lg:order-2' : ''">
          <h2
            class="max-w-md text-balance text-3xl font-semibold tracking-tight text-foreground-highlighted"
          >
            {{ feature.title }}
          </h2>
          <p class="mt-5 max-w-md text-base leading-7 text-foreground-muted">
            {{ feature.description }}
          </p>
          <NuxtLink
            :to="feature.link"
            class="mt-7 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted no-underline hover:underline underline-offset-4"
            >{{ feature.label }} <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
        </div>
        <figure
          :class="[
            'overflow-hidden rounded-xl border border-outline bg-muted',
            index % 2 ? 'lg:order-1' : '',
          ]"
        >
          <NuxtImg
            :src="feature.image"
            :alt="feature.alt"
            width="1440"
            height="900"
            sizes="100vw lg:600px"
            loading="lazy"
            class="block h-auto w-full"
          />
        </figure>
      </div>
    </section>

    <section class="bg-muted dark:bg-[#141414]">
      <div
        class="mx-auto grid w-full max-w-[82rem] gap-8 px-4 py-16 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:py-24"
      >
        <div>
          <h2 class="text-3xl font-semibold tracking-tight text-foreground-highlighted">
            The smaller things matter too.
          </h2>
          <p class="mt-4 max-w-md text-base leading-7 text-foreground-muted">
            Triage new work, track time against issues, and get back to the right page with a
            shortcut. These parts are already in the desktop app.
          </p>
        </div>
        <div class="grid gap-4 text-sm">
          <NuxtLink
            to="/issues/triage-and-archive"
            class="border-b border-outline py-3 text-foreground-highlighted no-underline hover:underline"
            >Triage and archive</NuxtLink
          >
          <NuxtLink
            to="/planning/time-tracking"
            class="border-b border-outline py-3 text-foreground-highlighted no-underline hover:underline"
            >Time tracking</NuxtLink
          >
          <NuxtLink
            to="/reference/keyboard-shortcuts"
            class="border-b border-outline py-3 text-foreground-highlighted no-underline hover:underline"
            >Keyboard shortcuts</NuxtLink
          >
        </div>
      </div>
    </section>
  </div>
</template>
