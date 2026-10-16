<script setup lang="ts">
import { ref, watch } from 'vue';
import { bulkTriage } from '../../lib/issue-storage';
import { notifications } from '../../lib/notifications';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import type { Issue } from '../../types/issue';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Select } from '../ui/select';
import { Textarea } from '../ui/textarea';

interface Props {
  issues: Issue[];
  decision: 'accept' | 'decline';
}

const props = defineProps<Props>();
const emit = defineEmits<{ completed: [] }>();
const open = defineModel<boolean>('open', { required: true });
const issueStore = useIssuesStore();

const reason = ref('');
const acceptedStatus = ref(issueStore.defaultStatus);
const busy = ref(false);

watch(open, (visible) => {
  if (!visible) return;
  reason.value = '';
  acceptedStatus.value = issueStore.defaultStatus;
});

async function confirm() {
  if (!props.issues.length || busy.value) return;

  busy.value = true;
  const selected = [...props.issues];
  const accept = props.decision === 'accept';
  const status = accept ? acceptedStatus.value : 'Canceled';

  try {
    if (usesNativeStorage) {
      const changed = await bulkTriage(
        selected.map((issue) => ({ identifier: issue.id, version: issue.version ?? 1 })),
        accept,
        status,
        reason.value.trim() || undefined,
      );
      issueStore.cacheRecords(changed);
      await issueStore.refreshCounts();
    } else {
      for (const issue of selected) {
        await issueStore.update(issue.id, { triage: false, status });
      }
    }

    open.value = false;
    reason.value = '';
    emit('completed');
    notifications.add({
      title: `${selected.length} ${selected.length === 1 ? 'issue' : 'issues'} ${accept ? 'accepted' : 'declined'}`,
      type: 'success',
    });
  } catch (error) {
    notifications.add({ title: 'Triage failed', description: String(error), type: 'error' });
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="`${decision === 'accept' ? 'Accept' : 'Decline'} ${issues.length} ${issues.length === 1 ? 'issue' : 'issues'}?`"
    description="This decision applies to every selected issue together."
    size="sm"
  >
    <template #body>
      <p class="text-xs text-foreground-muted">{{ issues.map((issue) => issue.id).join(', ') }}</p>
      <label
        v-if="decision === 'accept'"
        class="mt-4 block space-y-1.5 text-xs text-foreground-muted"
      >
        <span>Status after accepting</span>
        <Select
          v-model="acceptedStatus"
          :items="
            issueStore.statusGroups.filter(
              (status) => !['Completed', 'Canceled'].includes(issueStore.categoryFor(status)),
            )
          "
          aria-label="Status after accepting"
          size="sm"
        />
      </label>
      <label
        v-if="decision === 'decline'"
        class="mt-4 block space-y-1.5 text-xs text-foreground-muted"
      >
        <span>Reason (optional)</span>
        <Textarea
          v-model="reason"
          :rows="3"
          maxlength="1000"
          placeholder="Why is this issue being declined?"
        />
      </label>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="open = false" />
      <Button
        :label="decision === 'accept' ? 'Accept issues' : 'Decline issues'"
        :color="decision === 'decline' ? 'error' : 'info'"
        size="sm"
        :loading="busy"
        @click="confirm"
      />
    </template>
  </Dialog>
</template>
