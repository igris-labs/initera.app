import { groupDocsPages } from '../lib/docs-navigation';

export function useDocsNavigation() {
  return useAsyncData(
    'docs-navigation',
    () => queryCollectionNavigation('docs', ['description', 'icon']),
    { transform: groupDocsPages },
  );
}
