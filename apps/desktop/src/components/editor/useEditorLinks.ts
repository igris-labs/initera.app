import type { Editor } from '@tiptap/core';
import { ref } from 'vue';

export function normalizeLink(value: string): string | null {
  try {
    const url = new URL(value.trim());
    return ['http:', 'https:'].includes(url.protocol) ? url.toString() : null;
  } catch {
    return null;
  }
}

export function useEditorLinks(getEditor: () => Editor | undefined, isReadonly: () => boolean) {
  const linkDialogOpen = ref(false);
  const linkHref = ref('');
  const linkError = ref('');
  const linkCanRemove = ref(false);
  const linkSelection = ref<{ from: number; to: number } | null>(null);

  function openLinkDialog() {
    const current = getEditor();
    if (!current || isReadonly()) return;

    linkSelection.value = {
      from: current.state.selection.from,
      to: current.state.selection.to,
    };
    linkHref.value = current.getAttributes('link').href ?? '';
    linkCanRemove.value = current.isActive('link');
    linkError.value = '';
    linkDialogOpen.value = true;
  }

  function removeLink() {
    const current = getEditor();
    const selection = linkSelection.value;
    if (!current || !selection) return;

    current.chain().focus().setTextSelection(selection).unsetLink().run();
    linkDialogOpen.value = false;
  }

  function saveLink() {
    const current = getEditor();
    const selection = linkSelection.value;
    if (!current || !selection) return;

    const href = normalizeLink(linkHref.value);
    if (!href) {
      linkError.value = 'Enter a valid http or https link.';
      return;
    }

    const chain = current.chain().focus().setTextSelection(selection);
    if (selection.from === selection.to) {
      chain
        .insertContent({ type: 'text', text: href, marks: [{ type: 'link', attrs: { href } }] })
        .run();
    } else {
      chain.setLink({ href }).run();
    }

    linkDialogOpen.value = false;
  }

  return {
    linkDialogOpen,
    linkHref,
    linkError,
    linkCanRemove,
    openLinkDialog,
    removeLink,
    saveLink,
  };
}
