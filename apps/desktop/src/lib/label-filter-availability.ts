import type { LabelRecord } from './label-storage';
import type { IssueQuery } from '../types/issue';

export function unavailableLabelFilters(
  query: Pick<IssueQuery, 'labels' | 'excludedLabels' | 'projectLabels' | 'excludedProjectLabels'>,
  labels: readonly LabelRecord[],
): string[] {
  const available = new Set(
    labels.filter((label) => !label.deletedAt).map((label) => label.name.toLowerCase()),
  );
  return [
    ...new Set([
      ...query.labels,
      ...query.excludedLabels,
      ...query.projectLabels,
      ...query.excludedProjectLabels,
    ]),
  ].filter((name) => !available.has(name.toLowerCase()));
}
