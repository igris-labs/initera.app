import type { Priority, Status } from '../types/issue';

export { labelColor, labelSwatch } from './label-presentation';

export const statusOrder: Status[] = [
  'Todo',
  'In Progress',
  'On Hold',
  'Backlog',
  'Done',
  'Canceled',
];

export const statusIconChoices = [
  { name: 'i-tabler-circle-dashed', label: 'Backlog', group: 'Workflow' },
  { name: 'i-tabler-circle', label: 'Todo', group: 'Workflow' },
  { name: 'i-tabler-circle-dot', label: 'In progress', group: 'Workflow' },
  { name: 'i-tabler-circle-half-2', label: 'Half done', group: 'Workflow' },
  { name: 'i-tabler-circle-check', label: 'Done', group: 'Workflow' },
  { name: 'i-tabler-circle-x', label: 'Canceled', group: 'Workflow' },
  { name: 'i-tabler-player-pause', label: 'Paused', group: 'Workflow' },
  { name: 'i-tabler-hourglass', label: 'Waiting', group: 'Workflow' },
  { name: 'i-tabler-clock', label: 'Scheduled', group: 'Planning' },
  { name: 'i-tabler-calendar-time', label: 'Due soon', group: 'Planning' },
  { name: 'i-tabler-flag', label: 'Milestone', group: 'Planning' },
  { name: 'i-tabler-target', label: 'Goal', group: 'Planning' },
  { name: 'i-tabler-rocket', label: 'Launch', group: 'Planning' },
  { name: 'i-tabler-bolt', label: 'Urgent', group: 'Planning' },
  { name: 'i-tabler-flame', label: 'Hot', group: 'Planning' },
  { name: 'i-tabler-bulb', label: 'Idea', group: 'Planning' },
  { name: 'i-tabler-bug', label: 'Bug', group: 'Work' },
  { name: 'i-tabler-code', label: 'Code', group: 'Work' },
  { name: 'i-tabler-git-branch', label: 'Branch', group: 'Work' },
  { name: 'i-tabler-git-merge', label: 'Merge', group: 'Work' },
  { name: 'i-tabler-shield-check', label: 'Review', group: 'Work' },
  { name: 'i-tabler-message-circle', label: 'Discussion', group: 'Work' },
  { name: 'i-tabler-bookmark', label: 'Saved', group: 'Work' },
  { name: 'i-tabler-sparkles', label: 'Polish', group: 'Work' },
] as const;
export const statusIcons: readonly string[] = [
  ...statusIconChoices.map((choice) => choice.name),
  'i-tabler-circle-pause', // Legacy custom groups keep their saved icon.
];

export const statusPalette = {
  gray: 'text-foreground-muted',
  blue: 'text-(--label-blue)',
  purple: 'text-(--label-purple)',
  green: 'text-(--label-green)',
  amber: 'text-(--label-amber)',
  red: 'text-(--label-red)',
} as const;

export type StatusColor = keyof typeof statusPalette;

export const statusIcon: Record<Status, string> = {
  Backlog: 'i-tabler-circle-dashed',
  Todo: 'i-tabler-circle',
  'In Progress': 'i-tabler-circle-dot',
  'On Hold': 'i-tabler-player-pause',
  Done: 'i-tabler-circle-check-filled',
  Canceled: 'i-tabler-circle-x',
};

export const statusColor: Record<Status, string> = {
  Backlog: 'text-foreground-muted',
  Todo: 'text-foreground-muted',
  'In Progress': 'text-warning',
  'On Hold': 'text-(--label-purple)',
  Done: 'text-info',
  Canceled: 'text-foreground-dimmed',
};

export const priorityColor: Record<Priority, string> = {
  'No priority': 'text-foreground-dimmed',
  Low: 'text-foreground-muted',
  Medium: 'text-info',
  High: 'text-warning',
  Urgent: 'text-error',
};
