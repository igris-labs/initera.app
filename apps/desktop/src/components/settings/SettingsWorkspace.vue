<script setup lang="ts">
import { ref } from 'vue';
import { useWorkspaceStore } from '../../stores/workspace';
import { Button } from '../ui/button';
import { Input } from '../ui/input';

const workspace = useWorkspaceStore();
const userName = ref(workspace.userName);
const workspaceName = ref(workspace.workspaceName);

async function saveIdentity() {
  if (!userName.value.trim() || !workspaceName.value.trim()) return;
  try {
    await workspace.saveIdentity(userName.value, workspaceName.value);
  } catch {
    // Keep the edited values available for a retry.
  }
}
</script>

<template>
  <section aria-labelledby="workspace-heading">
    <h3 id="workspace-heading" class="text-sm font-medium text-foreground-highlighted">
      Workspace
    </h3>
    <form class="mt-3 grid gap-3 sm:grid-cols-2" @submit.prevent="saveIdentity">
      <label class="space-y-1 text-xs font-medium text-foreground-muted">
        Your name
        <Input v-model="userName" aria-label="Your name" maxlength="80" required size="md" />
      </label>
      <label class="space-y-1 text-xs font-medium text-foreground-muted">
        Workspace name
        <Input
          v-model="workspaceName"
          aria-label="Workspace name"
          maxlength="80"
          required
          size="md"
        />
      </label>
      <Button class="justify-self-end sm:col-span-2" label="Save names" type="submit" />
      <p v-if="workspace.error" class="text-xs text-danger sm:col-span-2" role="alert">
        {{ workspace.error }}
      </p>
    </form>
    <p v-if="workspace.dataRoot" class="mt-3 break-all text-xs text-foreground-muted">
      Data folder: {{ workspace.dataRoot }}
    </p>
  </section>
</template>
