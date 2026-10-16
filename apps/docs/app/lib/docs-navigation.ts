import type { ContentNavigationItem } from '@nuxt/content';

export interface DocsNavItem {
  title: string;
  path: string;
  description: string;
  icon: string;
  stem: string;
}

export interface DocsNavGroup {
  title: string;
  items: DocsNavItem[];
}

export function groupDocsPages(navigation: ContentNavigationItem[]): DocsNavGroup[] {
  const directories = new Map<string, DocsNavItem[]>();

  function visit(nodes: ContentNavigationItem[]) {
    for (const node of nodes) {
      const stem = String(node.stem ?? '');
      const directory = stem.split('/')[0];

      if (node.page !== false && directory && node.path && node.title) {
        const items = directories.get(directory) ?? [];
        items.push({
          title: node.title,
          path: node.path,
          description: String(node.description ?? ''),
          icon: String(node.icon ?? 'i-tabler-hash'),
          stem,
        });
        directories.set(directory, items);
      }

      if (node.children?.length) visit(node.children);
    }
  }

  visit(navigation);

  return [...directories]
    .sort(([a], [b]) => a.localeCompare(b, undefined, { numeric: true }))
    .map(([directory, items]) => {
      const name = directory.replace(/^\d+\./, '').replaceAll('-', ' ');

      return {
        title: name.charAt(0).toUpperCase() + name.slice(1),
        items,
      };
    });
}
