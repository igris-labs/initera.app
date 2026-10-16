<script setup lang="ts">
import { Node } from '@tiptap/core';
import { Markdown } from '@tiptap/markdown';
import { TaskItem, TaskList } from '@tiptap/extension-list';
import Placeholder from '@tiptap/extension-placeholder';
import StarterKit from '@tiptap/starter-kit';
import { EditorContent, useEditor } from '@tiptap/vue-3';
import { BubbleMenu } from '@tiptap/vue-3/menus';
import { watch } from 'vue';
import type { RichTextDocument } from '../../types/rich-text';
import { hydrateImages } from '../../lib/media-storage';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Listbox } from '../ui/listbox';
import { useSlashCommands } from './useSlashCommands';
import { normalizeLink, useEditorLinks } from './useEditorLinks';
import { useEditorImages } from './useEditorImages';

interface Props {
  modelValue: string;
  document?: RichTextDocument | null;
  mode?: 'title' | 'summary' | 'description';
  placeholder?: string;
  ariaLabel?: string;
  readonly?: boolean;
}

const {
  modelValue,
  document: editorDocument = null,
  mode = 'description',
  placeholder = '',
  ariaLabel = 'Rich text editor',
  readonly = false,
} = defineProps<Props>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:document': [value: RichTextDocument];
  'update:plainText': [value: string];
}>();

const { extension: slashCommands, slashMenu, selectedCommand } = useSlashCommands();

function highlightSlashCommand(value: unknown) {
  const index = slashMenu.value?.items.findIndex((item) => item.id === value) ?? -1;
  if (index >= 0) selectedCommand.value = index;
}

function onSlashHighlight(option?: { value?: unknown }) {
  highlightSlashCommand(option?.value);
}

function selectSlashCommand(value: unknown) {
  const item = slashMenu.value?.items.find((candidate) => candidate.id === value);
  if (item) slashMenu.value?.command(item);
}

const pendingLocalValues: string[] = [];
// Tiptap's JSON is authoritative for managed media; Markdown is a portable text fallback.
const inlineImage = Node.create({
  name: 'image',
  group: 'block',
  atom: true,
  addAttributes() {
    return { src: { default: null }, alt: { default: '' }, mediaId: { default: null } };
  },
  parseHTML() {
    return [{ tag: 'img[src]' }];
  },
  renderHTML({ HTMLAttributes }) {
    return ['img', HTMLAttributes];
  },
});

let editor: ReturnType<typeof useEditor>;
const { imageInput, imageError, addImage, insertImageFile } = useEditorImages(() => editor.value);
const { linkDialogOpen, linkHref, linkError, linkCanRemove, openLinkDialog, removeLink, saveLink } =
  useEditorLinks(
    () => editor.value,
    () => readonly,
  );

editor = useEditor({
  editable: !readonly,
  content: editorDocument?.version === 1 ? editorDocument.content : modelValue || '',
  contentType: editorDocument?.version === 1 ? undefined : 'markdown',
  extensions: [
    StarterKit.configure(
      mode !== 'description'
        ? {
            heading: false,
            blockquote: false,
            bulletList: false,
            orderedList: false,
            listItem: false,
            codeBlock: false,
            horizontalRule: false,
          }
        : { link: { openOnClick: readonly, isAllowedUri: (url) => normalizeLink(url) !== null } },
    ),
    Markdown,
    ...(mode === 'description' ? [TaskList, TaskItem.configure({ nested: true })] : []),
    ...(mode === 'description' ? [inlineImage] : []),
    Placeholder.configure({ placeholder }),
    ...(mode === 'description' && !readonly ? [slashCommands] : []),
  ],
  editorProps: {
    attributes: {
      role: 'textbox',
      'aria-label': ariaLabel,
      'aria-multiline': String(mode === 'description'),
      class: 'rich-text-content outline-none',
    },
    handleKeyDown: (_view, event) => {
      if (mode === 'description' && (event.metaKey || event.ctrlKey) && event.key === 'k') {
        event.preventDefault();
        openLinkDialog();
        return true;
      }

      if (mode !== 'description' && event.key === 'Enter') {
        event.preventDefault();
        (_view.dom.closest('form') as HTMLFormElement | null)?.requestSubmit();
        return true;
      }

      return false;
    },
    handlePaste: (_view, event) => {
      if (mode !== 'description' || readonly) return false;
      const image = Array.from(event.clipboardData?.files ?? []).find((file) =>
        file.type.startsWith('image/'),
      );
      if (!image) return false;
      event.preventDefault();
      insertImageFile(image);
      return true;
    },
  },
  onUpdate: ({ editor: current }) => {
    // The structured document keeps the media ID. Markdown is only a readable fallback.
    const markdown = current
      .getMarkdown()
      .replace(/data:image\/(?:png|jpeg|webp);base64,[A-Za-z0-9+/=]+/g, 'managed:image')
      .replace(/data:image\/svg\+xml,[^\s)]+/g, 'managed:image');
    pendingLocalValues.push(markdown);

    if (pendingLocalValues.length > 50) pendingLocalValues.shift();

    emit('update:modelValue', markdown);
    if (mode === 'description') emit('update:document', { version: 1, content: current.getJSON() });
    emit('update:plainText', current.getText().replace(/\s+/g, ' ').trim());
  },
});

function addBlock() {
  const current = editor.value;
  if (!current) return;

  const { $from } = current.state.selection;
  const at = $from.after(1);

  current
    .chain()
    .focus()
    .insertContentAt(at, { type: 'paragraph', content: [{ type: 'text', text: '/' }] })
    .setTextSelection(at + 2)
    .run();
}

// Parent updates can arrive after a newer local edit. Consume acknowledged values in order.
let contentRevision = 0;
watch(
  () => [modelValue, editorDocument, editor.value] as const,
  async ([value, nextDocument, current]) => {
    const localIndex = pendingLocalValues.indexOf(value);
    if (localIndex >= 0) {
      pendingLocalValues.splice(0, localIndex + 1);
      return;
    }

    pendingLocalValues.length = 0;
    if (!current) return;

    const revision = ++contentRevision;

    if (nextDocument?.version === 1) {
      // Image hydration is asynchronous; only the latest parent value may replace editor state.
      const hydrated = await hydrateImages(nextDocument);
      if (revision === contentRevision) {
        current.commands.setContent(hydrated.content, { emitUpdate: false });
      }
    } else if (value !== current.getMarkdown()) {
      current.commands.setContent(value || '', { contentType: 'markdown', emitUpdate: false });
    }
  },
  { immediate: true },
);

watch(
  () => readonly,
  (value) => editor.value?.setEditable(!value),
);

defineExpose({ focus: () => editor.value?.commands.focus() });
</script>

<template>
  <div :class="['rich-text-editor', `rich-text-${mode}`, readonly && 'rich-text-readonly']">
    <BubbleMenu
      v-if="editor && !readonly && mode === 'description'"
      :editor="editor"
      :update-delay="0"
      :options="{ placement: 'top', offset: 8, flip: true, shift: true }"
      class="rich-text-toolbar"
      aria-label="Formatting"
    >
      <button
        type="button"
        aria-label="Bold"
        title="Bold (⌘B)"
        :class="{ active: editor?.isActive('bold') }"
        @click="editor?.chain().focus().toggleBold().run()"
      >
        <Icon name="i-tabler-bold" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Italic"
        title="Italic (⌘I)"
        :class="{ active: editor?.isActive('italic') }"
        @click="editor?.chain().focus().toggleItalic().run()"
      >
        <Icon name="i-tabler-italic" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Heading"
        title="Heading"
        :class="{ active: editor?.isActive('heading', { level: 2 }) }"
        @click="editor?.chain().focus().toggleHeading({ level: 2 }).run()"
      >
        <Icon name="i-tabler-heading" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Bullet list"
        title="Bullet list"
        :class="{ active: editor?.isActive('bulletList') }"
        @click="editor?.chain().focus().toggleBulletList().run()"
      >
        <Icon name="i-tabler-list" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Numbered list"
        title="Numbered list"
        :class="{ active: editor?.isActive('orderedList') }"
        @click="editor?.chain().focus().toggleOrderedList().run()"
      >
        <Icon name="i-tabler-list-numbers" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Checklist"
        title="Checklist"
        :class="{ active: editor?.isActive('taskList') }"
        @click="editor?.chain().focus().toggleTaskList().run()"
      >
        <Icon name="i-tabler-list-check" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Quote"
        title="Quote"
        :class="{ active: editor?.isActive('blockquote') }"
        @click="editor?.chain().focus().toggleBlockquote().run()"
      >
        <Icon name="i-tabler-blockquote" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Code"
        title="Code"
        :class="{ active: editor?.isActive('code') }"
        @click="editor?.chain().focus().toggleCode().run()"
      >
        <Icon name="i-tabler-code" class="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Link"
        title="Link (⌘K)"
        :class="{ active: editor?.isActive('link') }"
        @pointerdown.prevent
        @mousedown.prevent
        @click="openLinkDialog"
      >
        <Icon name="i-tabler-link" class="size-3.5" />
      </button>
      <span class="rich-text-toolbar-divider" aria-hidden="true" />
      <button
        type="button"
        aria-label="Add block"
        title="Add block"
        @pointerdown.prevent
        @mousedown.prevent
        @click="addBlock"
      >
        <Icon name="i-tabler-plus" class="size-3.5" />
      </button>
    </BubbleMenu>
    <EditorContent :editor="editor" />
    <div v-if="mode === 'description' && !readonly" class="mt-2 flex items-center gap-2">
      <input
        ref="imageInput"
        type="file"
        accept="image/png,image/jpeg,image/gif,image/webp"
        class="sr-only"
        aria-label="Choose image"
        @change="addImage"
      />
      <Button
        label="Add image"
        icon="i-tabler-photo-plus"
        size="xs"
        variant="ghost"
        @click="imageInput?.click()"
      />
      <span v-if="imageError" role="alert" class="text-xs text-error">{{ imageError }}</span>
    </div>
    <Dialog v-model:open="linkDialogOpen" title="Add link" size="sm">
      <template #body>
        <form id="rich-text-link-form" class="space-y-2" @submit.prevent="saveLink">
          <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
            URL
            <Input
              v-model="linkHref"
              aria-label="Link URL"
              autofocus
              placeholder="https://example.com"
              size="sm"
              @update:model-value="linkError = ''"
            />
          </label>
          <p v-if="linkError" role="alert" class="text-xs text-error">{{ linkError }}</p>
        </form>
      </template>
      <template #footer>
        <Button
          v-if="linkCanRemove"
          label="Remove link"
          variant="ghost"
          size="xs"
          class="mr-auto"
          @click="removeLink"
        />
        <Button label="Cancel" variant="ghost" size="xs" @click="linkDialogOpen = false" />
        <Button label="Save link" size="xs" type="submit" form="rich-text-link-form" />
      </template>
    </Dialog>
    <Teleport v-if="slashMenu" :to="slashMenu.host">
      <div class="rich-text-slash-menu" @mousedown.prevent>
        <div class="rich-text-slash-heading">
          Insert block <span>/{{ slashMenu.query }}</span>
        </div>
        <Listbox
          aria-label="Insert block"
          :options="slashMenu.items.map((item) => ({ value: item.id, label: item.label }))"
          :model-value="slashMenu.items[selectedCommand]?.id ?? ''"
          :ui="{
            root: 'w-full',
            content: 'min-w-0 border-0 bg-transparent p-0 shadow-none',
            option:
              'min-h-10 gap-2 px-2 py-1.5 text-xs data-akaza-highlighted:bg-accented data-[state=checked]:bg-accented',
            empty: 'px-2 py-2 text-xs text-foreground-muted',
          }"
          @highlight-change="onSlashHighlight"
          @value-change="selectSlashCommand"
        >
          <template #option="{ value }">
            <span class="rich-text-slash-icon"
              ><Icon
                :name="slashMenu?.items.find((item) => item.id === value)?.icon ?? 'i-tabler-plus'"
                class="size-4"
            /></span>
            <span class="rich-text-slash-copy"
              ><strong>{{ slashMenu?.items.find((item) => item.id === value)?.label }}</strong
              ><small>{{
                slashMenu?.items.find((item) => item.id === value)?.description
              }}</small></span
            >
          </template>
          <template #empty>No matching blocks</template>
        </Listbox>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.rich-text-editor {
  color: var(--color-foreground);
}
.rich-text-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  z-index: 100;
  border: 1px solid var(--color-outline);
  border-radius: 8px;
  background: var(--color-elevated);
  padding: 3px;
  box-shadow: 0 8px 24px oklch(0 0 0 / 16%);
}
.rich-text-toolbar button {
  display: inline-flex;
  width: 26px;
  height: 26px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: var(--color-foreground-muted);
  cursor: pointer;
}
.rich-text-toolbar button:hover,
.rich-text-toolbar button.active {
  background: var(--color-accented);
  color: var(--color-foreground);
}
.rich-text-toolbar-divider {
  height: 16px;
  margin: 0 3px;
  border-left: 1px solid var(--color-outline);
}
.rich-text-title :deep(.rich-text-content) {
  min-height: 32px;
  font-size: 18px;
  line-height: 28px;
  font-weight: 600;
}
.rich-text-description :deep(.rich-text-content) {
  min-height: 80px;
  font-size: 14px;
  line-height: 21px;
}
.rich-text-summary :deep(.rich-text-content) {
  min-height: 24px;
  font-size: 14px;
  line-height: 20px;
}
.rich-text-slash-menu {
  max-height: min(360px, 50vh);
  overflow-y: auto;
  border: 1px solid var(--color-outline);
  border-radius: 10px;
  background: var(--color-elevated);
  padding: 5px;
  color: var(--color-foreground);
  box-shadow: 0 12px 32px oklch(0 0 0 / 18%);
}
.rich-text-slash-heading {
  display: flex;
  justify-content: space-between;
  padding: 6px 8px 7px;
  color: var(--color-foreground-muted);
  font-size: 12px;
  font-weight: 600;
}
.rich-text-slash-heading span {
  font-weight: 400;
}
.rich-text-slash-icon {
  display: inline-flex;
  width: 28px;
  height: 28px;
  flex: none;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-outline);
  border-radius: 6px;
  color: var(--color-foreground-muted);
}
.rich-text-slash-copy {
  display: grid;
  gap: 1px;
  min-width: 0;
}
.rich-text-slash-copy strong {
  font-size: 12px;
  font-weight: 500;
}
.rich-text-slash-copy small {
  font-size: 11px;
  color: var(--color-foreground-muted);
}
.rich-text-readonly :deep(.rich-text-content) {
  min-height: 0;
}
.rich-text-editor :deep(.rich-text-content p) {
  margin: 0 0 0.5em;
}
.rich-text-editor :deep(.rich-text-content p:last-child) {
  margin-bottom: 0;
}
.rich-text-editor :deep(.rich-text-content img) {
  display: block;
  max-width: min(100%, 640px);
  height: auto;
  margin: 0.75rem 0;
  border-radius: 0.5rem;
}
.rich-text-editor :deep(.rich-text-content h1),
.rich-text-editor :deep(.rich-text-content h2),
.rich-text-editor :deep(.rich-text-content h3) {
  margin: 0.8em 0 0.35em;
  font-weight: 600;
  color: var(--color-foreground);
}
.rich-text-editor :deep(.rich-text-content h1) {
  font-size: 1.35em;
}
.rich-text-editor :deep(.rich-text-content h2) {
  font-size: 1.2em;
}
.rich-text-editor :deep(.rich-text-content ul),
.rich-text-editor :deep(.rich-text-content ol) {
  margin: 0.4em 0;
  padding-left: 1.4em;
}
.rich-text-editor :deep(.rich-text-content ul) {
  list-style: disc;
}
.rich-text-editor :deep(.rich-text-content ol) {
  list-style: decimal;
}
.rich-text-editor :deep(.rich-text-content ul[data-type='taskList']) {
  list-style: none;
  padding-left: 0;
}
.rich-text-editor :deep(.rich-text-content ul[data-type='taskList'] li) {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}
.rich-text-editor :deep(.rich-text-content ul[data-type='taskList'] li > label) {
  flex: none;
  margin-top: 0.2rem;
}
.rich-text-editor :deep(.rich-text-content ul[data-type='taskList'] li input) {
  width: 0.875rem;
  height: 0.875rem;
  accent-color: var(--color-primary);
  cursor: pointer;
}
.rich-text-editor :deep(.rich-text-content ul[data-type='taskList'] li > div) {
  min-width: 0;
  flex: 1;
}
.rich-text-editor :deep(.rich-text-content blockquote) {
  margin: 0.5em 0;
  padding-left: 0.8em;
  border-left: 2px solid var(--color-outline);
  color: var(--color-foreground-muted);
}
.rich-text-editor :deep(.rich-text-content code) {
  border-radius: 4px;
  background: var(--color-muted);
  padding: 0.1em 0.3em;
  font-size: 0.9em;
}
.rich-text-editor :deep(.rich-text-content pre) {
  overflow: auto;
  border-radius: 8px;
  background: var(--color-muted);
  padding: 10px;
}
.rich-text-editor :deep(.rich-text-content pre code) {
  padding: 0;
}
.rich-text-editor :deep(.rich-text-content a) {
  color: var(--color-primary);
  text-decoration: underline;
}
.rich-text-editor :deep(.rich-text-content p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  height: 0;
  color: var(--color-foreground-dimmed);
  pointer-events: none;
}
</style>
