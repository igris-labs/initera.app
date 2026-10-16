<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { onMounted, ref } from 'vue';
import initeraSymbol from '../../assets/initera-symbol.svg';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { version as packageVersion } from '../../../package.json';
import licenseText from '../../../../../LICENSE?raw';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';

const version = ref(packageVersion);
const licenseOpen = ref(false);

onMounted(async () => {
  if (!usesNativeStorage) return;
  try {
    version.value = await getVersion();
  } catch {
    // The package version remains available if the native API cannot respond.
  }
});
</script>

<template>
  <section id="about" aria-labelledby="about-heading" class="border-t border-outline pt-6">
    <h3
      id="about-heading"
      class="flex items-center gap-2 text-sm font-medium text-foreground-highlighted"
    >
      <img :src="initeraSymbol" alt="" class="h-4 w-3 dark:brightness-0 dark:invert" />
      About Initera
    </h3>
    <dl class="mt-3 space-y-2 text-xs">
      <div class="flex items-baseline justify-between gap-4">
        <dt class="text-foreground-muted">Application version</dt>
        <dd class="font-mono text-foreground">{{ version }}</dd>
      </div>
      <div class="flex items-baseline justify-between gap-4">
        <dt class="text-foreground-muted">License</dt>
        <dd>
          <Button label="MIT License" variant="link" size="xs" @click="licenseOpen = true" />
        </dd>
      </div>
      <div class="flex items-baseline justify-between gap-4">
        <dt class="text-foreground-muted">Author</dt>
        <dd class="text-foreground">Rahul Vashishtha</dd>
      </div>
      <div class="flex items-baseline justify-between gap-4">
        <dt class="text-foreground-muted">Powered by</dt>
        <dd>
          <a
            href="https://igrislabs.com"
            target="_blank"
            rel="noopener noreferrer"
            class="text-foreground underline-offset-2 hover:underline"
          >
            Igris Labs
          </a>
        </dd>
      </div>
    </dl>
    <p class="mt-3 text-xs text-foreground-dimmed">
      MIT permits use, modification, and distribution. The software is provided without warranty.
    </p>

    <Dialog v-model:open="licenseOpen" title="MIT License" size="md">
      <template #body>
        <pre
          class="max-h-[60vh] overflow-auto font-sans text-xs leading-relaxed whitespace-pre-wrap text-foreground-muted"
          >{{ licenseText }}</pre>
      </template>
      <template #footer>
        <Button label="Close" size="sm" @click="licenseOpen = false" />
      </template>
    </Dialog>
  </section>
</template>
