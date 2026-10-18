<script setup lang="ts">
import {
  dismissUpdate,
  installUpdate,
  updateAvailable,
  updateError,
  updateInstalling,
  updateOpen,
  updateProgress,
} from '../../lib/app-updater';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';

function onOpenChange(open: boolean) {
  if (!open) dismissUpdate();
}
</script>

<template>
  <Dialog
    :open="updateOpen"
    :title="`Initera ${updateAvailable?.version ?? ''} is available`"
    size="sm"
    :close-on-backdrop-click="!updateInstalling"
    @update:open="onOpenChange"
  >
    <template #body>
      <p class="text-sm text-foreground-muted">Download and install the new version?</p>
      <p v-if="updateAvailable?.body" class="mt-3 whitespace-pre-wrap text-sm text-foreground">
        {{ updateAvailable.body }}
      </p>
      <p v-if="updateProgress" role="status" class="mt-3 text-xs text-foreground-muted">
        {{ updateProgress }}
      </p>
      <p v-if="updateError" role="alert" class="mt-3 text-xs text-danger">
        {{ updateError }}
      </p>
    </template>
    <template #footer>
      <Button
        label="Later"
        variant="ghost"
        size="sm"
        :disabled="updateInstalling"
        @click="dismissUpdate"
      />
      <Button
        label="Install update"
        size="sm"
        :disabled="updateInstalling"
        @click="installUpdate"
      />
    </template>
  </Dialog>
</template>
