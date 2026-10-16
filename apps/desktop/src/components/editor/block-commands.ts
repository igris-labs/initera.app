import type { Editor } from '@tiptap/core';

export interface BlockCommand {
  id:
    | 'paragraph'
    | 'heading2'
    | 'heading3'
    | 'bulletList'
    | 'orderedList'
    | 'taskList'
    | 'blockquote'
    | 'codeBlock'
    | 'horizontalRule';
  label: string;
  description: string;
  icon: string;
  keywords: string;
}

export const blockCommands: BlockCommand[] = [
  {
    id: 'paragraph',
    label: 'Text',
    description: 'Plain paragraph',
    icon: 'i-tabler-align-left',
    keywords: 'text paragraph',
  },
  {
    id: 'heading2',
    label: 'Heading 2',
    description: 'Medium section heading',
    icon: 'i-tabler-heading',
    keywords: 'heading h2 title',
  },
  {
    id: 'heading3',
    label: 'Heading 3',
    description: 'Small section heading',
    icon: 'i-tabler-heading',
    keywords: 'heading h3 title',
  },
  {
    id: 'bulletList',
    label: 'Bulleted list',
    description: 'List with bullets',
    icon: 'i-tabler-list',
    keywords: 'list bullet unordered',
  },
  {
    id: 'orderedList',
    label: 'Numbered list',
    description: 'List with numbers',
    icon: 'i-tabler-list-numbers',
    keywords: 'list numbered ordered',
  },
  {
    id: 'taskList',
    label: 'Checklist',
    description: 'Track items to complete',
    icon: 'i-tabler-list-check',
    keywords: 'task checklist checkbox todo',
  },
  {
    id: 'blockquote',
    label: 'Quote',
    description: 'Pull out a thought',
    icon: 'i-tabler-blockquote',
    keywords: 'quote blockquote',
  },
  {
    id: 'codeBlock',
    label: 'Code block',
    description: 'Code with fixed width text',
    icon: 'i-tabler-code',
    keywords: 'code snippet',
  },
  {
    id: 'horizontalRule',
    label: 'Divider',
    description: 'Separate sections',
    icon: 'i-tabler-minus',
    keywords: 'divider rule separator',
  },
];

export function searchBlockCommands(query: string): BlockCommand[] {
  const normalized = query.trim().toLowerCase();
  return blockCommands.filter((item) =>
    `${item.label} ${item.keywords}`.toLowerCase().includes(normalized),
  );
}

export function runBlockCommand(
  editor: Editor,
  range: { from: number; to: number },
  id: BlockCommand['id'],
) {
  const chain = editor.chain().focus().deleteRange(range);
  switch (id) {
    case 'paragraph':
      return chain.setParagraph().run();

    case 'heading2':
      return chain.setHeading({ level: 2 }).run();

    case 'heading3':
      return chain.setHeading({ level: 3 }).run();

    case 'bulletList':
      return chain.toggleBulletList().run();

    case 'orderedList':
      return chain.toggleOrderedList().run();

    case 'taskList':
      return chain.toggleTaskList().run();

    case 'blockquote':
      return chain.setBlockquote().run();

    case 'codeBlock':
      return chain.setCodeBlock().run();

    case 'horizontalRule':
      return chain.setHorizontalRule().run();
  }
}
