import { Extension } from '@tiptap/core';
import Suggestion from '@tiptap/suggestion';
import type { SuggestionProps } from '@tiptap/suggestion';
import { onScopeDispose, shallowRef } from 'vue';
import { runBlockCommand, searchBlockCommands } from './block-commands';
import type { BlockCommand } from './block-commands';

interface SlashMenuState {
  host: HTMLElement;
  items: BlockCommand[];
  query: string;
  command: (item: BlockCommand) => void;
}

export function useSlashCommands() {
  const slashMenu = shallowRef<SlashMenuState | null>(null);
  const selectedCommand = shallowRef(0);
  let unmountSlashMenu: (() => void) | undefined;

  onScopeDispose(() => unmountSlashMenu?.());

  function updateMenu(props: SuggestionProps<BlockCommand, BlockCommand>, host: HTMLElement) {
    slashMenu.value = { host, items: props.items, query: props.query, command: props.command };
    selectedCommand.value = 0;
  }

  const extension = Extension.create({
    name: 'workspaceSlashCommands',
    addProseMirrorPlugins() {
      return [
        Suggestion<BlockCommand, BlockCommand>({
          editor: this.editor,
          char: '/',
          container: '[role="dialog"][aria-modal="true"]',
          allow: ({ state, range }) => state.doc.resolve(range.from).parent.isTextblock,
          items: ({ query }) => searchBlockCommands(query),
          command: ({ editor, range, props: item }) => runBlockCommand(editor, range, item.id),
          render: () => ({
            onStart: (props) => {
              const host = document.createElement('div');
              host.style.zIndex = '100';
              host.style.width = '248px';
              unmountSlashMenu = props.mount(host);
              updateMenu(props, host);
            },
            onUpdate: (props) => {
              if (slashMenu.value) updateMenu(props, slashMenu.value.host);
            },
            onExit: () => {
              slashMenu.value = null;
              unmountSlashMenu?.();
              unmountSlashMenu = undefined;
            },
            onKeyDown: ({ event }) => {
              const items = slashMenu.value?.items ?? [];
              if (event.key === 'ArrowDown' && items.length) {
                selectedCommand.value = (selectedCommand.value + 1) % items.length;
                return true;
              }

              if (event.key === 'ArrowUp' && items.length) {
                selectedCommand.value = (selectedCommand.value - 1 + items.length) % items.length;
                return true;
              }

              if (event.key === 'Enter' && items.length) {
                slashMenu.value?.command(items[selectedCommand.value]);
                return true;
              }

              return false;
            },
          }),
        }),
      ];
    },
  });

  return { extension, slashMenu, selectedCommand };
}
