import type { Editor } from '@tiptap/core';
import { ref } from 'vue';
import { stageImage } from '../../lib/media-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';

export function useEditorImages(getEditor: () => Editor | undefined) {
  const imageInput = ref<HTMLInputElement | null>(null);
  const imageError = ref('');

  function addImage(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (file) void insertImageFile(file);
  }

  async function insertImageFile(file: File) {
    imageError.value = '';
    if (
      !(
        usesNativeStorage
          ? ['image/png', 'image/jpeg', 'image/webp']
          : ['image/png', 'image/jpeg', 'image/gif', 'image/webp']
      ).includes(file.type) ||
      file.size > (usesNativeStorage ? 20 * 1024 * 1024 : 1_000_000)
    ) {
      imageError.value = usesNativeStorage
        ? 'Choose a PNG, JPEG, or WebP image under 20 MB.'
        : 'Choose a PNG, JPEG, GIF, or WebP image under 1 MB.';
      return;
    }

    try {
      const source = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () =>
          typeof reader.result === 'string'
            ? resolve(reader.result)
            : reject(new Error('Could not read this image.'));
        reader.onerror = () => reject(new Error('Could not read this image.'));
        reader.readAsDataURL(file);
      });
      const media = usesNativeStorage ? await stageImage(source) : null;
      getEditor()
        ?.chain()
        .focus()
        .insertContent({
          type: 'image',
          attrs: {
            src: source,
            alt: file.name.replace(/\.[^.]+$/, ''),
            mediaId: media?.id ?? null,
          },
        })
        .run();
      if (imageInput.value) imageInput.value.value = '';
    } catch (error) {
      imageError.value = error instanceof Error ? error.message : String(error);
    }
  }

  return { imageInput, imageError, addImage, insertImageFile };
}
