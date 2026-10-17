<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'Initera — Keep work moving',
  description:
    'Plan projects, work through issues, and keep a clear record of progress in one desktop app.',
  ogTitle: 'Initera — Keep work moving',
  ogDescription: 'Projects, issues, cycles, and time tracking in one desktop app.',
});

const primaryLink = buttonVariants({ variant: 'solid', size: 'lg' });
const secondaryLink = buttonVariants({ variant: 'outline', size: 'lg' });
const shell = 'mx-auto w-full max-w-[82rem] px-4 sm:px-6';
const kicker = 'text-[0.7rem] font-semibold tracking-[0.14em] text-foreground-muted uppercase';
const heading =
  'mt-5 text-balance text-[clamp(2.5rem,4.2vw,4.3rem)] leading-[1.1] font-semibold tracking-[-0.065em] text-foreground-highlighted';
const textLink =
  'mt-4 inline-flex items-center gap-1.5 text-[0.8rem] font-semibold text-foreground-highlighted no-underline hover:underline underline-offset-4';
const imageFrame =
  'overflow-hidden rounded-[0.6rem] border border-outline bg-[#09090a] shadow-[0_3rem_6rem_-4rem_#000]';
const glow = ref<HTMLElement | null>(null);

let revealObserver: IntersectionObserver | undefined;
let glowAnimation: Animation | undefined;

onMounted(() => {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  const drift = () => Math.round((Math.random() - 0.5) * 64);
  glowAnimation = glow.value?.animate(
    [
      { transform: 'translate3d(0, 0, 0)' },
      { transform: `translate3d(${drift()}px, ${drift()}px, 0)` },
      { transform: `translate3d(${drift()}px, ${drift()}px, 0)` },
      { transform: 'translate3d(0, 0, 0)' },
    ],
    { duration: 36000, iterations: Infinity, easing: 'ease-in-out' },
  );

  document.querySelectorAll<HTMLElement>('[data-enter]').forEach((element, index) => {
    element.animate(
      [
        { opacity: 0, transform: 'translateY(14px)', filter: 'blur(5px)' },
        { opacity: 1, transform: 'translateY(0)', filter: 'blur(0)' },
      ],
      { duration: 700, delay: index * 90, easing: 'cubic-bezier(0.22, 1, 0.36, 1)' },
    );
  });

  if (!('IntersectionObserver' in window)) return;

  // Keep offscreen sections hidden before scroll; add transitions only when they enter.
  const hiddenClasses = ['opacity-0', 'translate-y-5', 'blur-[5px]'];

  revealObserver = new IntersectionObserver(
    (entries, observer) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;

        entry.target.classList.add(
          'transition-[opacity,translate,filter]',
          'duration-700',
          'ease-[cubic-bezier(0.22,1,0.36,1)]',
        );
        entry.target.classList.remove(...hiddenClasses);
        observer.unobserve(entry.target);
      }
    },
    { threshold: 0.12, rootMargin: '0px 0px -32px 0px' },
  );

  document.querySelectorAll<HTMLElement>('[data-reveal]').forEach((element) => {
    if (element.getBoundingClientRect().top < window.innerHeight - 32) return;

    element.classList.add(...hiddenClasses);
    revealObserver?.observe(element);
  });
});

onUnmounted(() => {
  revealObserver?.disconnect();
  glowAnimation?.cancel();
});
</script>

<template>
  <div class="overflow-hidden bg-default">
    <section
      :class="[shell, 'relative flex min-h-[38rem] items-center py-24 sm:py-32 lg:py-40']"
      aria-labelledby="hero-title"
    >
      <div
        aria-hidden="true"
        class="pointer-events-none absolute inset-x-0 top-32 flex justify-center"
      >
        <div
          ref="glow"
          class="h-96 w-[min(85vw,70rem)] rounded-full bg-black/[0.04] blur-[90px] dark:bg-white/[0.035]"
        />
      </div>
      <div class="relative max-w-[69rem]">
        <p :class="kicker" data-enter>PROJECTS · ISSUES · CYCLES</p>
        <h1
          id="hero-title"
          class="mt-6 max-w-[69rem] text-balance text-[clamp(2.4rem,10vw,3rem)] leading-[1.04] font-semibold tracking-[-0.08em] text-foreground-highlighted sm:text-[clamp(3.4rem,6.3vw,6.7rem)]"
          data-enter
        >
          Keep the work moving.<br /><span class="text-foreground-muted"
            >Keep the details close.</span
          >
        </h1>
        <p
          class="mt-8 max-w-[39rem] text-[clamp(1rem,1.35vw,1.12rem)] leading-relaxed text-foreground-muted"
          data-enter
        >
          Plan the project, pick the next issue, and see what changed. Initera keeps the work and
          its context together on your desktop.
        </p>
        <div class="mt-8 flex flex-wrap gap-3" data-enter>
          <NuxtLink
            to="/download"
            :class="[primaryLink, 'min-h-11 gap-2 px-4 text-[0.83rem] no-underline']"
          >
            Download Initera <Icon name="i-tabler-arrow-down" size="sm" />
          </NuxtLink>
          <NuxtLink
            to="/getting-started/quick-start"
            :class="[secondaryLink, 'min-h-11 px-4 text-[0.83rem] no-underline']"
          >
            Read the guide
          </NuxtLink>
        </div>
      </div>
      <div
        aria-hidden="true"
        class="absolute right-6 bottom-32 hidden font-mono text-[0.6rem] tracking-[0.12em] text-foreground-dimmed [writing-mode:vertical-rl] lg:block"
      >
        BUILT FOR THE DESKTOP / MADE FOR THE WORK
      </div>
    </section>

    <div :class="[shell, 'pb-28 lg:pb-44']" data-reveal>
      <figure :class="imageFrame">
        <NuxtImg
          src="/images/app/issues-list.png"
          alt="Initera issue list, grouped by status with priority, labels, and project context"
          width="1440"
          height="900"
          sizes="100vw lg:1280px"
          loading="eager"
          class="block h-auto w-[155%] max-w-none sm:w-full"
        />
      </figure>
      <p class="mt-3 text-left text-xs text-foreground-dimmed sm:text-right">
        An issue list with enough detail to decide what comes next.
      </p>
    </div>

    <section
      id="features"
      :class="[shell, 'grid scroll-mt-24 gap-14 pb-32 pt-12 lg:grid-cols-2 lg:gap-24 lg:pb-48']"
      aria-labelledby="features-title"
    >
      <div data-reveal>
        <p :class="kicker">THE WORK, IN CONTEXT</p>
        <h2 id="features-title" :class="heading">A place for every part of the plan.</h2>
      </div>
      <div data-reveal>
        <article class="grid grid-cols-[2rem_minmax(0,1fr)] gap-4 border-t border-outline py-7">
          <span class="pt-1 font-mono text-[0.7rem] text-foreground-dimmed">01</span>
          <div>
            <h3 class="text-lg font-semibold tracking-tight text-foreground-highlighted">Issues</h3>
            <p class="mt-2 max-w-md text-sm leading-relaxed text-foreground-muted">
              Track the task, its status, and who owns it. Add sub-issues or dependencies when one
              line is not enough.
            </p>
            <NuxtLink to="/issues/issues" :class="textLink"
              >Explore issues <Icon name="i-tabler-arrow-right" size="sm"
            /></NuxtLink>
          </div>
        </article>
        <article class="grid grid-cols-[2rem_minmax(0,1fr)] gap-4 border-t border-outline py-7">
          <span class="pt-1 font-mono text-[0.7rem] text-foreground-dimmed">02</span>
          <div>
            <h3 class="text-lg font-semibold tracking-tight text-foreground-highlighted">
              Projects
            </h3>
            <p class="mt-2 max-w-md text-sm leading-relaxed text-foreground-muted">
              Keep resources, milestones, updates, and linked issues beside the goal they serve.
            </p>
            <NuxtLink to="/projects/projects" :class="textLink"
              >Explore projects <Icon name="i-tabler-arrow-right" size="sm"
            /></NuxtLink>
          </div>
        </article>
        <article class="grid grid-cols-[2rem_minmax(0,1fr)] gap-4 border-y border-outline py-7">
          <span class="pt-1 font-mono text-[0.7rem] text-foreground-dimmed">03</span>
          <div>
            <h3 class="text-lg font-semibold tracking-tight text-foreground-highlighted">Cycles</h3>
            <p class="mt-2 max-w-md text-sm leading-relaxed text-foreground-muted">
              Give the next stretch of work a start and end. Review unfinished issues when the cycle
              closes.
            </p>
            <NuxtLink to="/planning/cycles" :class="textLink"
              >Explore cycles <Icon name="i-tabler-arrow-right" size="sm"
            /></NuxtLink>
          </div>
        </article>
      </div>
    </section>

    <section
      class="border-y border-outline bg-muted dark:bg-[#141414]"
      aria-labelledby="project-title"
    >
      <div
        :class="[
          shell,
          'grid items-center gap-14 py-24 lg:grid-cols-[0.7fr_1.1fr] lg:gap-28 lg:py-44',
        ]"
      >
        <div data-reveal>
          <p :class="kicker">PROJECTS</p>
          <h2 id="project-title" :class="[heading, 'max-w-[31rem]']">
            The reason behind the issue stays in view.
          </h2>
          <p class="mt-6 max-w-[31rem] text-[0.94rem] leading-relaxed text-foreground-muted">
            Open a project and see its description, milestones, resources, updates, and issue
            progress in one place.
          </p>
          <NuxtLink to="/projects/projects" :class="textLink"
            >See how projects work <Icon name="i-tabler-arrow-right" size="sm"
          /></NuxtLink>
        </div>
        <figure :class="imageFrame" data-reveal>
          <NuxtImg
            src="/images/app/project-overview.png"
            alt="Initera project overview with milestones, activity, and issue progress"
            width="1440"
            height="900"
            sizes="100vw lg:750px"
            loading="lazy"
            class="block h-auto min-h-64 w-full object-cover object-left-top lg:min-h-[27rem]"
          />
        </figure>
      </div>
    </section>

    <section
      id="workflow"
      :class="[shell, 'scroll-mt-24 py-32 lg:py-48']"
      aria-labelledby="workflow-title"
    >
      <div class="max-w-[49rem]" data-reveal>
        <p :class="kicker">STAY IN FLOW</p>
        <h2 id="workflow-title" :class="heading">
          Find the next move without leaving the keyboard.
        </h2>
        <p class="mt-6 max-w-[31rem] text-[0.94rem] leading-relaxed text-foreground-muted">
          Search for an issue or project, jump to a view, and run actions from the command bar.
        </p>
        <NuxtLink to="/reference/command-bar" :class="textLink"
          >See the command bar <Icon name="i-tabler-arrow-right" size="sm"
        /></NuxtLink>
      </div>
      <figure :class="[imageFrame, 'mx-auto mt-16 max-w-[70rem]']" data-reveal>
        <NuxtImg
          src="/images/app/command-bar.png"
          alt="Initera command bar with search and actions available from the keyboard"
          width="1440"
          height="900"
          sizes="100vw lg:1100px"
          loading="lazy"
          class="block h-auto w-full"
        />
      </figure>
    </section>

    <section
      :class="[
        shell,
        'flex min-h-[34rem] flex-col items-center justify-center border-t border-outline py-24 text-center',
      ]"
      aria-labelledby="final-title"
      data-reveal
    >
      <p :class="kicker">READY WHEN YOU ARE</p>
      <h2 id="final-title" :class="[heading, 'mb-8']">
        Start with one issue.<br />Build from there.
      </h2>
      <NuxtLink
        to="/download"
        :class="[primaryLink, 'inline-flex min-h-11 gap-2 px-4 text-[0.83rem] no-underline']"
      >
        Download Initera <Icon name="i-tabler-arrow-down" size="sm" />
      </NuxtLink>
    </section>
  </div>
</template>
