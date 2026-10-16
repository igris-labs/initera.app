import { parseDate } from '@internationalized/date';
import type { CalendarDate } from '@internationalized/date';
import { watchDebounced } from '@vueuse/core';
import { computed, onScopeDispose, ref, watch } from 'vue';
import type { ComputedRef } from 'vue';
import { registerDraftFlusher } from '../lib/pending-writes';
import { useIssuesStore } from '../stores/issues';
import { useWorkspaceStore } from '../stores/workspace';
import type { Issue } from '../types/issue';
import type { RichTextDocument } from '../types/rich-text';
import { useRecoveryDraft } from './useRecoveryDraft';

interface IssueEditDraft {
  title: string;
  titleMarkdown: string;
  description: string;
  descriptionDocument: RichTextDocument | null;
  startDate: string | null;
  dueDate: string | null;
}

export function useIssueDetailEditor(issue: ComputedRef<Issue>) {
  const issues = useIssuesStore();
  const workspace = useWorkspaceStore();
  const editing = ref(false);
  const saving = ref(false);
  const editError = ref('');
  const editState = ref<'saved' | 'saving' | 'failed'>('saved');
  const titleDraft = ref('');
  const titleMarkdownDraft = ref('');
  const descriptionDraft = ref('');
  const descriptionDocumentDraft = ref<RichTextDocument | null>(null);
  const startDateDraft = ref<CalendarDate | undefined>();
  const dueDateDraft = ref<CalendarDate | undefined>();
  const dueDateEdited = ref(false);

  const {
    value: recoveryDraft,
    waitUntilReady: waitForRecoveryDraft,
    discard: discardRecoveryDraft,
  } = useRecoveryDraft<IssueEditDraft>(
    computed(() => `issue-edit:${workspace.workspaceId}:${issue.value.id}`),
  );

  const currentDraft = computed<IssueEditDraft>(() => ({
    title: titleDraft.value,
    titleMarkdown: titleMarkdownDraft.value,
    description: descriptionDraft.value,
    descriptionDocument: descriptionDocumentDraft.value,
    startDate: startDateDraft.value?.toString() ?? null,
    dueDate: dueDateDraft.value?.toString() ?? null,
  }));

  let savedSignature = '';
  let saveChain = Promise.resolve();
  let inFlightSignature = '';
  let saveCount = 0;

  watch(currentDraft, (draft) => {
    if (editing.value && JSON.stringify(draft) !== savedSignature) recoveryDraft.value = draft;
  });

  watchDebounced(
    currentDraft,
    () => {
      if (editing.value) void saveEdit();
    },
    { debounce: 500, deep: true },
  );

  async function beginEdit() {
    await waitForRecoveryDraft();
    const draft = recoveryDraft.value;
    const current = issue.value;
    titleDraft.value = draft?.title ?? current.title;
    titleMarkdownDraft.value = draft?.titleMarkdown ?? current.titleMarkdown ?? current.title;
    descriptionDraft.value = draft?.description ?? current.description;
    descriptionDocumentDraft.value =
      draft?.descriptionDocument ?? current.descriptionDocument ?? null;
    const recoveredStart = draft ? draft.startDate : current.startDate;
    const recoveredDue = draft ? draft.dueDate : current.due;
    startDateDraft.value = recoveredStart ? parseDate(recoveredStart) : undefined;
    dueDateDraft.value =
      recoveredDue && /^\d{4}-\d{2}-\d{2}$/.test(recoveredDue)
        ? parseDate(recoveredDue)
        : undefined;
    dueDateEdited.value = Boolean(draft);
    editError.value = '';
    editState.value = 'saved';
    savedSignature = JSON.stringify({
      title: current.title,
      titleMarkdown: current.titleMarkdown || current.title,
      description: current.description,
      descriptionDocument: current.descriptionDocument ?? null,
      startDate: current.startDate ?? null,
      dueDate: current.due ?? null,
    });
    editing.value = true;
  }

  async function saveEdit(closeAfter = false, failOnError = false) {
    const draft = currentDraft.value;
    const dueWasEdited = dueDateEdited.value;
    const signature = JSON.stringify(draft);
    if (signature === inFlightSignature) {
      await saveChain;
      if (closeAfter && !recoveryDraft.value) editing.value = false;
      return;
    }

    if (signature === savedSignature) {
      if (closeAfter) editing.value = false;
      return;
    }

    if (
      startDateDraft.value &&
      dueDateDraft.value &&
      startDateDraft.value.toString() > dueDateDraft.value.toString()
    ) {
      editError.value = 'Due date must be on or after the start date.';
      if (failOnError) throw new Error(editError.value);
      return;
    }

    recoveryDraft.value = draft;
    editState.value = 'saving';
    saving.value = true;
    saveCount++;
    inFlightSignature = signature;
    // Serialize autosaves so an earlier request cannot overwrite a newer draft.
    const previous = saveChain;
    const task = previous.then(async () => {
      await issues.save(issue.value.id, {
        title: draft.title,
        titleMarkdown: draft.titleMarkdown,
        description: draft.description,
        descriptionDocument: draft.descriptionDocument,
        startDate: draft.startDate,
        due: dueWasEdited ? (draft.dueDate ?? undefined) : issue.value.due,
      });
    });
    saveChain = task.catch(() => {});
    try {
      await task;
      savedSignature = signature;
      if (JSON.stringify(currentDraft.value) === signature) {
        await discardRecoveryDraft();
        editState.value = 'saved';
        if (closeAfter) editing.value = false;
      }

      editError.value = '';
    } catch (error) {
      editError.value = String(error);
      editState.value = 'failed';
      if (failOnError) throw error;
    } finally {
      saveCount--;
      saving.value = saveCount > 0;
      if (inFlightSignature === signature) inFlightSignature = '';
    }
  }

  const unregisterEditFlusher = registerDraftFlusher(async () => {
    if (!editing.value) return;
    await saveEdit(false, true);
    if (editState.value === 'failed')
      throw new Error(editError.value || 'Issue changes were not saved.');
  });
  onScopeDispose(unregisterEditFlusher);

  function onEditBlur(event: FocusEvent) {
    if (!(event.currentTarget as HTMLElement).contains(event.relatedTarget as Node | null))
      void saveEdit();
  }

  return {
    editing,
    saving,
    editError,
    editState,
    titleDraft,
    titleMarkdownDraft,
    descriptionDraft,
    descriptionDocumentDraft,
    startDateDraft,
    dueDateDraft,
    dueDateEdited,
    beginEdit,
    saveEdit,
    onEditBlur,
  };
}
