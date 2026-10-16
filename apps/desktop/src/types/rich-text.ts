import type { JSONContent } from '@tiptap/core';

/** Versioned Tiptap document; Markdown remains the readable fallback. */
export interface RichTextDocument {
  version: 1;
  content: JSONContent;
}
