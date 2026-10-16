import { invoke } from './native-invoke';
import type { JSONContent } from '@tiptap/core';
import type { RichTextDocument } from '../types/rich-text';
import { usesNativeStorage } from './workspace-storage';

interface ManagedImage {
  id: string;
  mime: string;
  width: number;
  height: number;
}

const missingImageSource = `data:image/svg+xml,${encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" width="320" height="120" viewBox="0 0 320 120"><rect x="1" y="1" width="318" height="118" rx="8" fill="none" stroke="#92929b" stroke-opacity=".4"/><path d="M143 38h34v32h-34zM143 64l10-10 7 7 5-4 12 13" fill="none" stroke="#92929b" stroke-width="2" stroke-linejoin="round"/><text x="160" y="91" fill="#92929b" font-family="sans-serif" font-size="14" text-anchor="middle">Image unavailable</text></svg>',
)}`;

export async function stageImage(source: string): Promise<ManagedImage> {
  return invoke<ManagedImage>('storage_stage_image', { source });
}

async function imageSource(id: string): Promise<string> {
  return invoke<string>('storage_read_image', { id });
}

export async function hydrateImages(document: RichTextDocument): Promise<RichTextDocument> {
  if (!usesNativeStorage) return document;

  // Renderer URLs are temporary display data. Do not write them into the saved document.
  const copy = structuredClone(document);
  const images: JSONContent[] = [];

  function collect(node: JSONContent) {
    if (node.type === 'image' && typeof node.attrs?.mediaId === 'string') images.push(node);
    node.content?.forEach(collect);
  }

  collect(copy.content);
  await Promise.all(
    images.map(async (node) => {
      try {
        node.attrs = { ...node.attrs, src: await imageSource(node.attrs?.mediaId as string) };
      } catch {
        node.attrs = { ...node.attrs, src: missingImageSource };
      }
    }),
  );

  return copy;
}
