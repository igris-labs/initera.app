export const issueTableColumns = {
  identifier: { label: 'ID', width: 104, min: 72, max: 180 },
  title: { label: 'Title', width: 360, min: 180, max: 800 },
  status: { label: 'Status', width: 120, min: 92, max: 240 },
  priority: { label: 'Priority', width: 110, min: 86, max: 220 },
  labels: { label: 'Labels', width: 180, min: 100, max: 360 },
  project: { label: 'Project', width: 160, min: 100, max: 320 },
  node: { label: 'Node', width: 150, min: 100, max: 320 },
  milestone: { label: 'Milestone', width: 150, min: 100, max: 320 },
  cycle: { label: 'Cycle', width: 150, min: 100, max: 320 },
  estimate: { label: 'Estimate', width: 100, min: 78, max: 180 },
  dueDate: { label: 'Due date', width: 130, min: 100, max: 220 },
  trackedTime: { label: 'Tracked', width: 110, min: 86, max: 180 },
  updatedAt: { label: 'Updated', width: 110, min: 90, max: 200 },
} as const;

export type IssueTableColumn = keyof typeof issueTableColumns;
export type IssueTableWidths = Partial<Record<IssueTableColumn, number>>;

export function visibleIssueColumns(
  columns: string[] | undefined,
  showPriority: boolean,
  showProject: boolean,
): IssueTableColumn[] {
  const requested = columns ?? [
    'identifier',
    'title',
    'status',
    'priority',
    'labels',
    'project',
    'updatedAt',
  ];
  const visible = requested
    .filter((column): column is IssueTableColumn => column in issueTableColumns)
    .filter(
      (column) => (column !== 'priority' || showPriority) && (column !== 'project' || showProject),
    );
  return [
    'identifier',
    'title',
    ...visible.filter((column) => column !== 'identifier' && column !== 'title'),
  ];
}

export function issueTableTemplate(columns: IssueTableColumn[], widths: IssueTableWidths = {}) {
  return columns
    .map((column) => {
      const config = issueTableColumns[column];
      const width = Math.max(config.min, Math.min(config.max, widths[column] ?? config.width));
      return column === 'title' ? `minmax(${width}px, 1fr)` : `${width}px`;
    })
    .join(' ');
}
