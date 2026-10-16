import type { Issue } from '../types/issue';

export function collectDescendantIds(
  items: readonly Pick<Issue, 'id' | 'parentId'>[],
  rootId: string,
): Set<string> {
  const children = new Map<string, string[]>();
  for (const item of items) {
    if (!item.parentId) continue;
    const siblings = children.get(item.parentId) ?? [];
    siblings.push(item.id);
    children.set(item.parentId, siblings);
  }

  const descendants = new Set([rootId]);
  const pending = [rootId];
  while (pending.length) {
    for (const id of children.get(pending.pop()!) ?? []) {
      // The visited set also protects preview data imported with a malformed cycle.
      if (descendants.has(id)) continue;
      descendants.add(id);
      pending.push(id);
    }
  }

  return descendants;
}
