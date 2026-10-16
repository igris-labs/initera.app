<script setup lang="ts">
import { ref, watch } from 'vue';
import type { IssueMovePreview } from '../../lib/issue-storage';
import { useIssuesStore } from '../../stores/issues';
import { getProjectMovePreference, setProjectMovePreference } from '../../lib/workspace-options';
import { notifications } from '../../lib/notifications';
import type { ProjectMovePreference } from '../../lib/workspace-options';
import { Button } from '../ui/button';
import { Checkbox } from '../ui/checkbox';
import { Dialog } from '../ui/dialog';
import { RadioGroup } from '../ui/radio-group';

interface Props {
  identifier: string;
  destination: string;
}

const { identifier, destination } = defineProps<Props>();
const emit = defineEmits<{ moved: [previousId: string, nextId: string] }>();
const open = defineModel<boolean>('open', { required: true });
const issues = useIssuesStore();
const preview = ref<IssueMovePreview | null>(null);
const policy = ref<'keep' | 'regenerate'>('keep');
const error = ref('');
const busy = ref(false);
const preference = ref<ProjectMovePreference>('ask');
const remember = ref(false);

watch(
  () => [open.value, identifier, destination] as const,
  async ([visible, issueId, projectName]) => {
    preview.value = null;
    error.value = '';
    remember.value = false;
    if (!visible || !issueId || !projectName) return;

    try {
      preference.value = await getProjectMovePreference();
      policy.value = preference.value === 'regenerate' ? 'regenerate' : 'keep';
      preview.value = await issues.previewProjectMove(issueId, projectName);
    } catch (cause) {
      error.value = String(cause);
    }
  },
  { immediate: true },
);

async function confirm() {
  if (!preview.value) return;
  busy.value = true;
  error.value = '';

  try {
    const previous = preview.value.identifier;
    const next = await issues.moveProject(preview.value, policy.value);
    open.value = false;
    emit('moved', previous, next);

    if (remember.value && preference.value === 'ask') {
      try {
        await setProjectMovePreference(policy.value);
      } catch (cause) {
        notifications.add({
          title: 'Issue moved',
          description: `Move preference could not be saved: ${String(cause)}`,
          type: 'warning',
        });
      }
    }
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="`Move ${identifier} to ${destination}`"
    description="Review identifiers and project-specific links before moving this issue."
    size="sm"
  >
    <template #body>
      <p v-if="!preview && !error" class="text-sm text-foreground-muted">Checking the move…</p>
      <template v-if="preview">
        <p class="text-sm text-foreground-muted">
          {{ preview.affectedCount }} issue{{ preview.affectedCount === 1 ? '' : 's' }} will move.
        </p>
        <ul class="mt-3 space-y-1 text-xs text-foreground-muted">
          <li v-if="preview.clearsMilestones">
            {{ preview.clearsMilestones }} milestone assignment{{
              preview.clearsMilestones === 1 ? '' : 's'
            }}
            will clear.
          </li>
          <li v-if="preview.clearsNodes">
            {{ preview.clearsNodes }} module or feature assignment{{
              preview.clearsNodes === 1 ? '' : 's'
            }}
            will clear.
          </li>
          <li v-if="preview.clearsCycles">
            {{ preview.clearsCycles }} incompatible cycle assignment{{
              preview.clearsCycles === 1 ? '' : 's'
            }}
            will clear.
          </li>
        </ul>
        <RadioGroup
          :model-value="policy"
          :options="[
            {
              value: 'keep',
              label: `Keep ${identifier}`,
              description: 'The issue keeps its current ID.',
            },
            {
              value: 'regenerate',
              label: `Use ${preview.proposedIdentifier}`,
              description:
                'Proposed number may change before commit. Previous IDs remain searchable.',
            },
          ]"
          aria-label="Issue identifier after move"
          size="sm"
          class="mt-4"
          @value-change="policy = $event as 'keep' | 'regenerate'"
        />
        <label
          v-if="preference === 'ask'"
          class="mt-3 flex items-center gap-2 text-xs text-foreground-muted"
        >
          <Checkbox v-model="remember" size="sm" /> Remember this choice for this workspace
        </label>
      </template>
      <p v-if="error" role="alert" class="mt-3 text-xs text-error">{{ error }}</p>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="open = false" />
      <Button label="Move issue" size="sm" :disabled="!preview || busy" @click="confirm" />
    </template>
  </Dialog>
</template>
