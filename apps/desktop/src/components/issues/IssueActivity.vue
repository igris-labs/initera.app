<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { findComment, listIssueStatusVisits } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import type { IssueActivity as ActivityEntry, StatusVisit } from '../../types/issue';
import type { RichTextDocument } from '../../types/rich-text';
import { useIssuesStore } from '../../stores/issues';
import { useWorkspaceStore } from '../../stores/workspace';
import { useRecoveryDraft } from '../../composables/useRecoveryDraft';
import { Button } from '../ui/button';
import { RichTextEditor } from '../editor';

interface Props {
  issueId: string;
  expanded?: boolean;
}

const props = defineProps<Props>();

const issues = useIssuesStore();
const route = useRoute();

const activity = computed(() => issues.activityByIssue[props.issueId] ?? []);
const statusVisits = ref<StatusVisit[]>([]);

const commentDraft = ref('');
const commentDocument = ref<RichTextDocument | null>(null);
const commentPlainText = ref('');
const postingComment = ref(false);
const error = ref('');
const focusedComment = ref<ActivityEntry | null>(null);
const focusedMissing = ref(false);
const editingId = ref<string | null>(null);
const deletingId = ref<string | null>(null);
const editDraft = ref('');
const editDocument = ref<RichTextDocument | null>(null);
const editPlainText = ref('');
const {
  value: recoveryComment,
  waitUntilReady: waitForRecoveryComment,
  discard: discardRecoveryComment,
} = useRecoveryDraft<{
  text: string;
  document: RichTextDocument | null;
}>(computed(() => `issue-comment:${useWorkspaceStore().workspaceId}:${props.issueId}`));

watch([commentDraft, commentDocument], ([text, document]) => {
  recoveryComment.value = text || document ? { text, document } : null;
});

watch(
  () => props.issueId,
  async (id) => {
    await waitForRecoveryComment();
    error.value = '';
    commentDraft.value = recoveryComment.value?.text ?? '';
    commentDocument.value = recoveryComment.value?.document ?? null;
    commentPlainText.value = recoveryComment.value?.text ?? '';
    void issues.loadActivity(id).catch((cause) => {
      error.value = String(cause);
    });
    if (usesNativeStorage) {
      void listIssueStatusVisits(id)
        .then((visits) => {
          if (id === props.issueId) statusVisits.value = visits;
        })
        .catch((cause) => {
          error.value = String(cause);
        });
    }
  },
  { immediate: true },
);

let focusRevision = 0;

watch(
  [() => props.issueId, () => route.query.comment],
  async ([issueId, commentId]) => {
    // Deep-link lookup can finish after navigation; only the latest target may scroll.
    const revision = ++focusRevision;
    focusedComment.value = null;
    focusedMissing.value = false;
    if (typeof commentId !== 'string' || !commentId) return;

    try {
      await issues.loadActivity(issueId);
      const entry = usesNativeStorage
        ? await findComment(issueId, commentId)
        : (issues.activityByIssue[issueId]?.find((item) => item.id === commentId) ?? null);
      if (revision !== focusRevision) return;

      focusedComment.value = entry;
      focusedMissing.value = !entry;
      await nextTick();
      document.getElementById(`activity-${commentId}`)?.scrollIntoView({ block: 'center' });
    } catch (cause) {
      if (revision === focusRevision) error.value = String(cause);
    }
  },
  { immediate: true },
);

function activityDate(value: string) {
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(
    new Date(value),
  );
}

function visitDuration(visit: StatusVisit) {
  const elapsed = Math.max(
    0,
    Date.parse(visit.leftAt ?? new Date().toISOString()) - Date.parse(visit.enteredAt),
  );
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ${minutes % 60}m`;
  return `${Math.floor(hours / 24)}d ${hours % 24}h`;
}

async function postComment() {
  if (!commentDraft.value.trim() && !commentDocument.value) return;

  postingComment.value = true;
  error.value = '';
  try {
    await issues.addComment(
      props.issueId,
      commentDocument.value ? commentPlainText.value.trim() || 'Image' : commentDraft.value,
      commentDocument.value,
    );
    commentDraft.value = '';
    commentDocument.value = null;
    commentPlainText.value = '';
    await discardRecoveryComment();
  } catch (cause) {
    error.value = String(cause);
  } finally {
    postingComment.value = false;
  }
}

function commentContent(entry: ActivityEntry): RichTextDocument | null {
  if (!entry.bodyDocumentJson) return null;

  try {
    const document = JSON.parse(entry.bodyDocumentJson) as RichTextDocument;
    return document.version === 1 ? document : null;
  } catch {
    return null;
  }
}

function beginEdit(entry: ActivityEntry) {
  editingId.value = entry.id;
  deletingId.value = null;
  editDraft.value = entry.body;
  editPlainText.value = entry.body;
  editDocument.value = commentContent(entry);
  error.value = '';
}

async function saveComment(entry: ActivityEntry) {
  try {
    await issues.editComment(
      props.issueId,
      entry,
      editDocument.value ? editPlainText.value.trim() || 'Image' : editDraft.value,
      editDocument.value,
    );
    editingId.value = null;
  } catch (cause) {
    error.value = String(cause);
  }
}

async function removeComment(entry: ActivityEntry) {
  try {
    await issues.removeComment(props.issueId, entry);
    deletingId.value = null;
  } catch (cause) {
    error.value = String(cause);
  }
}

async function loadOlderActivity() {
  try {
    await issues.loadActivity(props.issueId, true);
  } catch (cause) {
    error.value = String(cause);
  }
}
</script>

<template>
  <section
    :class="
      expanded
        ? 'min-w-0 border-t border-outline pt-5 lg:col-start-1 lg:row-start-2'
        : 'space-y-3 border-t border-outline pt-4'
    "
  >
    <h3 class="px-1 text-sm font-medium text-foreground-highlighted">Activity</h3>
    <p v-if="error" role="alert" class="px-1 text-xs text-error">{{ error }}</p>

    <details v-if="statusVisits.length" class="px-1 text-xs text-foreground-muted">
      <summary class="cursor-pointer">Status history · {{ statusVisits.length }} visits</summary>
      <ol class="mt-2 space-y-1 pl-3">
        <li v-for="(visit, index) in statusVisits" :key="`${visit.enteredAt}:${index}`">
          {{ visit.statusName }} · {{ activityDate(visit.enteredAt) }} ·
          {{ visit.leftAt ? visitDuration(visit) : 'current' }}
        </li>
      </ol>
    </details>

    <form class="mt-3 rounded-md border border-outline p-2" @submit.prevent="postComment">
      <RichTextEditor
        v-model="commentDraft"
        v-model:document="commentDocument"
        aria-label="Write a comment"
        placeholder="Write a comment..."
        @update:plain-text="commentPlainText = $event"
      />
      <div class="mt-2 flex justify-end">
        <Button
          label="Post comment"
          type="submit"
          size="xs"
          :disabled="postingComment || (!commentDraft.trim() && !commentDocument)"
        />
      </div>
    </form>

    <ol class="mt-4 space-y-4">
      <li
        v-if="focusedComment && !activity.some((item) => item.id === focusedComment?.id)"
        :id="`activity-${focusedComment.id}`"
        class="rounded-md bg-accented px-3 py-2 text-xs"
      >
        <p class="mb-1 text-foreground-dimmed">Matched comment in older activity</p>
        <p class="text-foreground-muted">
          {{ focusedComment.actorName }} · {{ activityDate(focusedComment.createdAt) }}
        </p>
        <RichTextEditor
          :model-value="focusedComment.body"
          :document="commentContent(focusedComment)"
          readonly
          aria-label="Matched comment content"
          class="mt-1 text-sm"
        />
      </li>
      <li
        v-for="entry in activity"
        :id="`activity-${entry.id}`"
        :key="entry.id"
        class="flex gap-2 px-1 text-xs"
        :class="entry.id === focusedComment?.id ? 'rounded-md bg-accented py-2' : ''"
      >
        <span
          class="inline-flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/15 text-primary"
          >{{ entry.actorName.charAt(0).toUpperCase() }}</span
        >
        <div class="min-w-0 flex-1">
          <p class="flex flex-wrap items-baseline gap-x-1">
            <span class="font-medium text-foreground-highlighted">{{ entry.actorName }}</span>
            <span class="text-foreground-muted">{{
              entry.kind === 'comment' ? 'commented' : entry.body
            }}</span>
            <span class="text-foreground-dimmed">· {{ activityDate(entry.createdAt) }}</span>
          </p>
          <p v-if="entry.deletedAt" class="mt-1 italic text-foreground-dimmed">Comment deleted</p>
          <RichTextEditor
            v-else-if="entry.kind === 'comment' && editingId !== entry.id"
            :model-value="entry.body"
            :document="commentContent(entry)"
            readonly
            aria-label="Comment content"
            class="mt-1 text-sm"
          />
          <div v-if="editingId === entry.id" class="mt-2 rounded-md border border-outline p-2">
            <RichTextEditor
              v-model="editDraft"
              v-model:document="editDocument"
              aria-label="Edit comment"
              @update:plain-text="editPlainText = $event"
            />
            <div class="mt-2 flex gap-1">
              <Button label="Save comment" size="xs" @click="saveComment(entry)" />
              <Button label="Cancel" size="xs" variant="ghost" @click="editingId = null" />
            </div>
          </div>
          <div v-else-if="deletingId === entry.id" class="mt-1 flex items-center gap-1">
            <span class="text-foreground-muted">Delete this comment?</span>
            <Button label="Delete" size="xs" variant="ghost" @click="removeComment(entry)" />
            <Button label="Cancel" size="xs" variant="ghost" @click="deletingId = null" />
          </div>
          <div v-else-if="entry.kind === 'comment' && !entry.deletedAt" class="mt-1 flex gap-1">
            <Button label="Edit" size="xs" variant="ghost" @click="beginEdit(entry)" />
            <Button label="Delete" size="xs" variant="ghost" @click="deletingId = entry.id" />
            <span v-if="entry.editedAt" class="self-center text-foreground-dimmed">Edited</span>
          </div>
        </div>
      </li>
    </ol>
    <p v-if="focusedMissing" class="mt-3 px-1 text-xs text-foreground-dimmed">
      This comment is no longer available.
    </p>

    <Button
      v-if="issues.activityHasMore[issueId]"
      label="Load older activity"
      variant="ghost"
      size="xs"
      class="mt-3"
      @click="loadOlderActivity"
    />
  </section>
</template>
