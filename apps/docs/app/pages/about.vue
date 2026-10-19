<script setup lang="ts">
definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'About Initera',
  description: 'See how Initera keeps projects, issues, cycles, and time records together.',
});
</script>

<template>
  <div class="mx-auto w-full max-w-[82rem] px-4 py-20 sm:px-6 lg:py-28">
    <div class="max-w-[46rem]">
      <p class="text-[0.7rem] font-semibold tracking-[0.14em] text-foreground-muted uppercase">
        ABOUT INITERA
      </p>
      <h1
        class="mt-7 text-balance text-[clamp(2.2rem,5vw,3.5rem)] leading-[1.08] font-semibold tracking-[-0.07em] text-foreground-highlighted"
      >
        See the whole project, down to the next issue.
      </h1>
      <p class="mt-7 max-w-[42rem] text-base leading-relaxed text-foreground-muted">
        Open a project to see its milestones, linked issues, and updates. Open an issue to see its
        status, related work, and recent activity. You can keep that context on your computer
        without creating an online account.
      </p>
    </div>

    <div
      class="mt-16 grid items-start gap-12 border-t border-outline pt-12 lg:grid-cols-[1.1fr_0.9fr] lg:gap-20"
    >
      <figure class="overflow-hidden rounded-[0.6rem] border border-outline bg-muted">
        <NuxtImg
          src="/images/app/project-overview.png"
          alt="Initera project overview with milestones, updates, and linked issues"
          width="1440"
          height="900"
          sizes="100vw lg:720px"
          class="block h-auto w-full"
        />
      </figure>
      <div class="space-y-8">
        <section>
          <h2 class="text-lg font-semibold tracking-tight text-foreground-highlighted">
            Keep the plan with the work
          </h2>
          <p class="mt-3 text-sm leading-relaxed text-foreground-muted">
            Give a project its milestones, resources, and updates. Link issues to it so you can see
            the work behind each goal.
          </p>
        </section>
        <section>
          <h2 class="text-lg font-semibold tracking-tight text-foreground-highlighted">
            Make the next step clear
          </h2>
          <p class="mt-3 text-sm leading-relaxed text-foreground-muted">
            Break issues into smaller tasks, connect related work, and plan a cycle. Search and
            saved views help you get back to what matters today.
          </p>
        </section>
        <section>
          <h2 class="text-lg font-semibold tracking-tight text-foreground-highlighted">
            Keep your own copy
          </h2>
          <p class="mt-3 text-sm leading-relaxed text-foreground-muted">
            Your work lives on your computer. Create a backup or export a copy when you need one.
            Initera is open source under the MIT License.
          </p>
        </section>
        <div class="flex flex-wrap gap-x-6 gap-y-2 text-xs font-semibold">
          <NuxtLink
            to="/getting-started/quick-start"
            class="text-foreground-highlighted underline underline-offset-4"
          >
            Read the guide
          </NuxtLink>
          <NuxtLink to="/download" class="text-foreground-highlighted underline underline-offset-4">
            Download Initera
          </NuxtLink>
        </div>
      </div>
    </div>

    <div class="mt-20 border-t border-outline pt-8 text-sm text-foreground-muted">
      <p class="font-medium text-foreground-highlighted">About the author</p>
      <p class="mt-2">
        Created by
        <a
          href="https://rahulv.dev"
          target="_blank"
          rel="noopener noreferrer"
          class="underline underline-offset-4"
          >Rahul Vashishtha</a
        >
        at
        <a
          href="https://igrislabs.com"
          target="_blank"
          rel="noopener noreferrer"
          class="underline underline-offset-4"
          >Igris Labs</a
        >.
      </p>
    </div>
  </div>
</template>
