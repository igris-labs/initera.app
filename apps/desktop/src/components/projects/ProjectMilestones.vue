<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
  loadRemovedMilestones,
  restoreMilestone,
  setMilestoneArchived,
} from '../../lib/workspace-storage';
import type { ProjectRecord, RemovedMilestone } from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import { useProjectsStore } from '../../stores/projects';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';

interface Props {
  project: ProjectRecord;
}

const { project } = defineProps<Props>();
const emit = defineEmits<{ edit: [] }>();
const issues = useIssuesStore();
const projects = useProjectsStore();
const removedMilestones = ref<RemovedMilestone[]>([]);
const milestoneError = ref('');
const restoreTarget = ref<RemovedMilestone | null>(null);
const restoreOpen = ref(false);

watch(
  () => project.id,
  async (id) => {
    removedMilestones.value = await loadRemovedMilestones(id).catch(() => []);
  },
  { immediate: true },
);

const milestoneIssueCounts = computed(() => {
  if (issues.counts) return issues.counts.projects[project.name]?.milestones ?? {};
  const counts: Record<string, { total: number; done: number }> = {};
  for (const issue of issues.items) {
    if (issue.project !== project.name || issue.triage || !issue.milestoneId) continue;
    const count = (counts[issue.milestoneId] ??= { total: 0, done: 0 });
    count.total += 1;
    if (issues.categoryFor(issue.status) === 'Completed') count.done += 1;
  }

  return counts;
});

function openMilestoneRestore(milestone: RemovedMilestone) {
  restoreTarget.value = milestone;
  restoreOpen.value = true;
}

async function restoreRemovedMilestone(reassignIssues: boolean) {
  const target = restoreTarget.value;
  if (!target) return;
  try {
    await restoreMilestone(project.id, target.id, reassignIssues, project.version ?? 0);
    await projects.refreshOne(project.id);
    removedMilestones.value = await loadRemovedMilestones(project.id);
    milestoneError.value = '';
    restoreOpen.value = false;
    restoreTarget.value = null;
  } catch (error) {
    milestoneError.value = String(error);
  }
}

async function toggleMilestoneArchive(milestoneId: string, archived: boolean) {
  try {
    await setMilestoneArchived(project.id, milestoneId, archived, project.version ?? 0);
    await projects.refreshOne(project.id);
    milestoneError.value = '';
  } catch (cause) {
    milestoneError.value = String(cause);
  }
}
</script>

<template>
  <section aria-label="Project milestones" class="border-t border-outline pt-6">
    <div class="mb-3 flex items-center justify-between">
      <h3 class="text-xs font-medium text-foreground-muted">
        Milestones
        <span class="ml-1 tabular-nums text-foreground-dimmed">{{
          project.milestones.length
        }}</span>
      </h3>
      <Button
        label="Add milestone"
        icon="i-tabler-plus"
        size="xs"
        variant="ghost"
        @click="emit('edit')"
      />
    </div>
    <div v-if="project.milestones.length" class="space-y-1">
      <div
        v-for="milestone in project.milestones"
        :key="milestone.id"
        class="flex items-center gap-1"
      >
        <button
          type="button"
          class="flex min-w-0 flex-1 items-start gap-3 rounded-md px-2 py-2 text-left transition-colors hover:bg-accented/60"
          @click="emit('edit')"
        >
          <Icon name="i-tabler-diamond" class="mt-0.5 size-4 shrink-0 text-foreground-muted" />
          <span class="min-w-0 flex-1"
            ><span class="block text-xs font-medium"
              >{{ milestone.title }}
              <span v-if="milestone.archivedAt" class="text-foreground-dimmed"
                >· Archived</span
              ></span
            ><span
              v-if="milestone.description"
              class="mt-0.5 block text-xs text-foreground-muted"
              >{{ milestone.description }}</span
            ></span
          >
          <span class="shrink-0 text-xs tabular-nums text-foreground-muted">
            {{ milestoneIssueCounts[milestone.id]?.done ?? 0 }}/{{
              milestoneIssueCounts[milestone.id]?.total ?? 0
            }}
            issues
          </span>
          <span v-if="milestone.targetDate" class="shrink-0 text-xs text-foreground-muted">{{
            milestone.targetDate
          }}</span>
        </button>
        <Button
          :label="milestone.archivedAt ? 'Restore' : 'Archive'"
          size="xs"
          variant="ghost"
          :aria-label="`${milestone.archivedAt ? 'Restore' : 'Archive'} milestone ${milestone.title}`"
          @click="toggleMilestoneArchive(milestone.id, !milestone.archivedAt)"
        />
      </div>
    </div>
    <p v-else class="text-xs text-foreground-dimmed">Break the project into smaller stages.</p>
    <div v-if="removedMilestones.length" class="mt-4 border-t border-outline pt-3">
      <p class="mb-2 text-xs text-foreground-muted">Removed milestones</p>
      <div
        v-for="milestone in removedMilestones"
        :key="milestone.id"
        class="flex items-center gap-2 py-1 text-xs"
      >
        <span class="min-w-0 flex-1 truncate">{{ milestone.title }}</span>
        <Button
          label="Restore"
          size="xs"
          variant="ghost"
          @click="openMilestoneRestore(milestone)"
        />
      </div>
    </div>
    <p v-if="milestoneError" role="alert" class="mt-2 text-xs text-error">
      {{ milestoneError }}
    </p>
  </section>
  <Dialog v-model:open="restoreOpen" title="Restore milestone" size="sm">
    <template #body>
      <p class="text-sm text-foreground-muted">
        Restore {{ restoreTarget?.title }}. {{ restoreTarget?.detachedIssueCount ?? 0 }} issues were
        assigned when it was removed.
      </p>
      <p class="mt-2 text-xs text-foreground-dimmed">
        Reassignment applies only to issues still in this project without another milestone.
      </p>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="restoreOpen = false" />
      <Button
        label="Restore only"
        variant="soft"
        size="sm"
        @click="restoreRemovedMilestone(false)"
      />
      <Button
        v-if="restoreTarget?.detachedIssueCount"
        label="Restore and reassign"
        size="sm"
        @click="restoreRemovedMilestone(true)"
      />
    </template>
  </Dialog>
</template>
