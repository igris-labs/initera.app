<script setup lang="ts">
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'About Initera',
  description:
    'Initera is a local desktop project management app by Rahul Vashishtha at Igris Labs.',
});
</script>

<template>
  <div class="mx-auto min-h-[65vh] w-full max-w-[82rem] px-4 py-24 sm:px-6 lg:py-36">
    <div class="max-w-[54rem]">
      <p class="text-[0.7rem] font-semibold tracking-[0.14em] text-foreground-muted uppercase">
        ABOUT INITERA
      </p>
      <h1
        class="mt-7 max-w-[55rem] text-balance text-[clamp(3.2rem,13vw,5rem)] leading-none font-semibold tracking-[-0.085em] text-foreground-highlighted sm:text-[clamp(3.5rem,6.5vw,6.5rem)]"
      >
        Made by Rahul Vashishtha.
      </h1>
      <p class="mt-8 max-w-[42rem] text-[1.05rem] leading-relaxed text-foreground-muted">
        Initera is a desktop project manager from Igris Labs, Rahul's personal laboratory. It puts
        projects, issues, cycles, and time records in one place on your computer.
      </p>
    </div>

    <div class="mt-20 grid gap-14 border-t border-outline pt-12 lg:grid-cols-[0.8fr_1fr] lg:gap-24">
      <div
        class="grid min-h-72 place-items-center rounded-[0.6rem] border border-outline bg-muted lg:min-h-[26rem]"
        aria-hidden="true"
      >
        <img
          src="/initera-symbol.svg"
          alt=""
          class="w-[min(30%,9rem)] dark:brightness-0 dark:invert"
        />
      </div>
      <div>
        <h2 class="text-[1.7rem] font-semibold tracking-[-0.05em] text-foreground-highlighted">
          What it keeps together.
        </h2>
        <p class="mt-5 max-w-[39rem] text-sm leading-loose text-foreground-muted">
          Start with an issue. Connect it to a project when it needs a larger plan. Add a cycle when
          work needs a time frame. The app keeps the record close to the work, including updates,
          comments, and time entries.
        </p>
        <p class="mt-5 max-w-[39rem] text-sm leading-loose text-foreground-muted">
          Initera uses Vue, Tauri, and SQLite. Your work stays on your computer unless you export a
          backup. The project is available under the MIT License.
        </p>
        <NuxtLink
          to="/reference/about"
          class="mt-4 inline-flex items-center gap-1.5 text-xs font-semibold text-foreground-highlighted no-underline hover:underline underline-offset-4"
        >
          Product and license details <Icon name="i-tabler-arrow-right" size="sm" />
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
