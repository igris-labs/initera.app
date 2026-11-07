<script setup lang="ts">
import { Accordion } from '../ui/accordion';

const questions = [
  {
    value: 'account',
    label: 'Do I need an account?',
    content:
      'No. Create a workspace in the desktop app and start using it. Accounts and sync are planned, but are not part of the current release.',
  },
  {
    value: 'data',
    label: 'Where is my data stored?',
    content:
      'Initera keeps workspace records in a local SQLite database and stores images as files alongside it. You can find the data folder in workspace settings.',
  },
  {
    value: 'backup',
    label: 'Can I move my work to another computer?',
    content:
      'Yes. Export a backup, transfer the file, and import it on the other computer. The backup includes the database and its images.',
  },
  {
    value: 'license',
    label: 'Can I use Initera at work?',
    content:
      'Yes. Initera is open source under the MIT License. Check your organization’s policies before installing software on a managed computer.',
  },
];
</script>

<template>
  <section class="bg-muted dark:bg-[#141414]" aria-labelledby="faq-heading">
    <div
      class="mx-auto grid w-full max-w-[82rem] gap-10 px-4 py-20 sm:px-6 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20 lg:py-28"
    >
      <div>
        <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">
          Questions
        </p>
        <h2
          id="faq-heading"
          class="mt-5 text-3xl font-semibold tracking-tight text-foreground-highlighted sm:text-4xl"
        >
          Before you start
        </h2>
      </div>
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
    </div>
  </section>
</template>
