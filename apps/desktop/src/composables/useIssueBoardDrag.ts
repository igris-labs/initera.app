import Sortable from 'sortablejs';
import { nextTick, onBeforeUnmount, onMounted, watch } from 'vue';
import type { ComputedRef, Ref } from 'vue';
import type { Issue } from '../types/issue';

type GroupBy = 'status' | 'priority';

interface BoardGroup {
  name: string;
  items: Issue[];
}

interface BoardDragOptions {
  root: Ref<HTMLElement | null>;
  groups: ComputedRef<BoardGroup[]>;
  groupBy: () => GroupBy | 'project' | 'none';
  archivedGroups: () => string[];
  move: (id: string, property: GroupBy, value: string, index: number) => void;
  reorderGroup: (property: GroupBy, name: string, index: number) => void;
}

export function useIssueBoardDrag(options: BoardDragOptions) {
  const sortables: Sortable[] = [];
  let suppressClickUntil = 0;

  function destroySortables() {
    sortables.splice(0).forEach((sortable) => sortable.destroy());
  }

  function onGroupDrop(event: Sortable.SortableEvent) {
    const name = (event.item as HTMLElement).dataset.boardGroup;
    if (!name || !options.root.value) return;

    // Sortable moves the element itself. Restore it before Vue applies the new order.
    event.item.remove();
    options.root.value.insertBefore(
      event.item,
      options.root.value.children[event.oldIndex ?? 0] ?? null,
    );
    const property = options.groupBy();
    if (property === 'status' || property === 'priority')
      options.reorderGroup(property, name, event.newDraggableIndex ?? 0);
  }

  function onCardDrop(event: Sortable.SortableEvent) {
    suppressClickUntil = performance.now() + 200;
    const id = (event.item as HTMLElement).dataset.issueId;
    const lane = (event.to as HTMLElement).dataset.boardLane;
    if (!id || !lane) return;

    // Vue owns final position. Sortable animates peers during drag.
    event.item.remove();
    event.from.insertBefore(event.item, event.from.children[event.oldIndex ?? 0] ?? null);
    const offset = Number((event.to as HTMLElement).dataset.boardOffset ?? 0);
    const property = options.groupBy();
    if (property === 'status' || property === 'priority')
      options.move(id, property, lane, offset + (event.newDraggableIndex ?? 0));
  }

  function setupSortables() {
    destroySortables();
    if (!options.root.value || options.groupBy() === 'project' || options.groupBy() === 'none')
      return;

    const easing = getComputedStyle(document.documentElement)
      .getPropertyValue('--ipex-ease-enter')
      .trim();

    options.root.value.querySelectorAll<HTMLElement>('[data-board-lane]').forEach((element) => {
      sortables.push(
        Sortable.create(element, {
          group: {
            name: 'issues-board',
            pull: true,
            put: !options.archivedGroups().includes(element.dataset.boardLane ?? ''),
          },
          draggable: '[data-issue-id]',
          delay: 140,
          touchStartThreshold: 4,
          forceFallback: true,
          fallbackOnBody: true,
          fallbackTolerance: 3,
          animation: 120,
          easing,
          ghostClass: 'opacity-40',
          chosenClass: 'ring-1',
          onEnd: onCardDrop,
        }),
      );
    });

    sortables.push(
      Sortable.create(options.root.value, {
        draggable: '[data-board-group]',
        handle: '[data-board-group-handle]',
        direction: 'horizontal',
        delay: 140,
        touchStartThreshold: 4,
        forceFallback: true,
        fallbackOnBody: true,
        fallbackTolerance: 3,
        animation: 120,
        easing,
        ghostClass: 'opacity-40',
        onEnd: onGroupDrop,
      }),
    );
  }

  onMounted(setupSortables);
  watch(
    () => `${options.groupBy()}:${options.groups.value.map((group) => group.name).join('|')}`,
    setupSortables,
    { flush: 'post' },
  );
  onBeforeUnmount(destroySortables);

  function canOpenIssue() {
    return performance.now() >= suppressClickUntil;
  }

  async function moveWithKeyboard(event: KeyboardEvent, issue: Issue) {
    const property = options.groupBy();
    if (property !== 'status' && property !== 'priority') return;
    if (!event.altKey || !event.key.startsWith('Arrow')) return;
    const groupIndex = options.groups.value.findIndex((group) => group.name === issue[property]);
    const group = options.groups.value[groupIndex];
    if (!group) return;

    const itemIndex = group.items.findIndex((item) => item.id === issue.id);
    const nextGroup =
      event.key === 'ArrowLeft' || event.key === 'ArrowRight'
        ? options.groups.value[groupIndex + (event.key === 'ArrowRight' ? 1 : -1)]
        : group;
    if (!nextGroup) return;

    const nextIndex =
      nextGroup === group
        ? itemIndex + (event.key === 'ArrowDown' ? 1 : -1)
        : nextGroup.items.length;
    if (nextGroup === group && (nextIndex < 0 || nextIndex >= group.items.length)) return;

    event.preventDefault();
    options.move(issue.id, property, nextGroup.name, nextIndex);
    await nextTick();
    options.root.value?.querySelector<HTMLElement>(`[data-issue-id="${issue.id}"]`)?.focus();
  }

  async function moveGroupWithKeyboard(event: KeyboardEvent, name: string) {
    const property = options.groupBy();
    if (property !== 'status' && property !== 'priority') return;
    if (options.archivedGroups().includes(name)) return;
    if (!event.altKey || !['ArrowLeft', 'ArrowRight'].includes(event.key)) return;

    const index = options.groups.value.findIndex((group) => group.name === name);
    const next = index + (event.key === 'ArrowRight' ? 1 : -1);
    if (next < 0 || next >= options.groups.value.length) return;

    event.preventDefault();
    const handle = event.currentTarget as HTMLElement;
    options.reorderGroup(property, name, next);
    await nextTick();
    handle.focus();
  }

  return { canOpenIssue, moveWithKeyboard, moveGroupWithKeyboard };
}
