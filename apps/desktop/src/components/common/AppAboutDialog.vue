<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { onMounted, ref } from 'vue';
import initeraSymbol from '../../assets/initera-symbol.svg';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { checkForUpdates, updateChecking } from '../../lib/app-updater';
import { version as packageVersion } from '../../../package.json';
import licenseText from '../../../../../LICENSE?raw';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';

const version = ref(packageVersion);
const open = defineModel<boolean>('open', { required: true });
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
  <Dialog v-model:open="open" title="About Initera" size="sm">
    <template #body>
      <img :src="initeraSymbol" alt="Initera" class="mb-4 h-9 w-7 dark:brightness-0 dark:invert" />
      <dl class="space-y-2 text-xs">
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
    </template>
    <template #footer>
      <Button label="Close" variant="ghost" size="sm" @click="open = false" />
      <Button
        v-if="usesNativeStorage"
        label="Check for updates"
        size="sm"
        :disabled="updateChecking"
        @click="checkForUpdates(true)"
      />
    </template>
  </Dialog>

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
</template>
