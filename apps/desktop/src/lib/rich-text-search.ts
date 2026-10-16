import type { JSONContent } from '@tiptap/core';
import type { RichTextDocument } from '../types/rich-text';

const blockTypes = new Set([
  'doc',
  'blockquote',
  'bulletList',
  'orderedList',
  'listItem',
  'taskList',
  'taskItem',
]);

function textFromNode(node: JSONContent): string {
  if (node.type === 'hardBreak') return ' ';
  if (node.type === 'text') return node.text ?? '';

  const text = (node.content ?? []).map(textFromNode);
  return text.join(blockTypes.has(node.type ?? '') ? ' ' : '');
}

export function searchableDescription(
  document: RichTextDocument | null | undefined,
  markdown: string,
): string {
  if (document?.version !== 1) return markdown;
  return textFromNode(document.content).replace(/\s+/g, ' ').trim();
}
