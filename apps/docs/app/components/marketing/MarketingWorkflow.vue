<script setup lang="ts">
import { Icon } from '../ui/icon';

const steps = [
  {
    number: '01',
    eyebrow: 'Capture',
    title: 'Write the issue while the detail is fresh.',
    description:
      'Add a title, status, priority, and description. Assign a project, split out sub-issues, and link related work when the task grows.',
    image: '/images/app/issue-detail.png',
    alt: 'Issue detail showing properties, description, and activity',
    link: '/issues/issues',
    linkLabel: 'Work with issues',
  },
  {
    number: '02',
    eyebrow: 'Organize',
    title: 'Give that work a project to belong to.',
    description:
      'Use milestones to mark the path. Keep resources, updates, and linked issues together so the project page shows what is moving.',
    image: '/images/app/project-overview.png',
    alt: 'Project overview with milestones and linked issues',
    link: '/projects/projects',
    linkLabel: 'Plan a project',
  },
  {
    number: '03',
    eyebrow: 'Plan',
    title: 'Decide what fits in the next cycle.',
    description:
      'Set a cycle schedule, assign issues, and review unfinished work when the period ends. Track time on an issue when a duration matters.',
    image: '/images/app/cycles.png',
    alt: 'Cycles page with a schedule and current work',
    link: '/planning/cycles',
    linkLabel: 'Plan with cycles',
  },
];
</script>

<template>
  <section class="scroll-mt-24" aria-labelledby="workflow-heading">
    <div class="mx-auto w-full max-w-[82rem] px-4 pt-20 sm:px-6 lg:pt-28">
      <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
        One connected workflow
      </p>
      <h2
        id="workflow-heading"
        class="mt-5 max-w-[46rem] text-balance text-3xl leading-tight font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
      >
        From a loose task to a plan you can follow.
      </h2>
      <p class="mt-5 max-w-[43rem] text-base leading-7 text-foreground-muted">
        Start small. Add structure when the work needs it. Each issue stays connected to its
        project, cycle, and history.
      </p>
    </div>

    <article v-for="(step, index) in steps" :key="step.number">
      <div
        class="mx-auto grid w-full max-w-[82rem] items-center gap-12 px-4 py-20 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:py-28"
      >
        <div :class="['max-w-[32rem]', index % 2 === 1 ? 'lg:order-2' : '']">
          <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
            {{ step.number }} / {{ step.eyebrow }}
          </p>
          <h3
            class="mt-5 text-balance text-3xl leading-tight font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
          >
            {{ step.title }}
          </h3>
          <p class="mt-5 text-base leading-7 text-foreground-muted">{{ step.description }}</p>
          <NuxtLink
            :to="step.link"
            class="mt-7 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted no-underline hover:underline underline-offset-4"
            >{{ step.linkLabel }} <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
        </div>
        <figure
          :class="[
            'overflow-hidden rounded-xl border border-outline bg-muted shadow-[0_2rem_5rem_-3rem_#000]',
            index % 2 === 1 ? 'lg:order-1' : '',
          ]"
        >
          <NuxtImg
            :src="step.image"
            :alt="step.alt"
            width="1440"
            height="900"
            sizes="100vw lg:600px"
            loading="lazy"
            class="block aspect-[1.4] w-full object-cover object-left-top sm:aspect-auto sm:h-auto"
          />
        </figure>
      </div>
    </article>
  </section>
</template>
