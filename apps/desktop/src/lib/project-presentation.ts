import { priorities } from '../types/issue';
import { priorityColor, statusColor, statusIcon } from './issue-presentation';
import { defaultProjectIcon, iconColor } from './icon-presentation';
import type { ProjectRecord } from './workspace-storage';

export const projectStatuses = [
  {
    label: 'Backlog',
    value: 'Backlog',
    icon: statusIcon.Backlog,
    iconClass: statusColor.Backlog,
  },
  {
    label: 'Planned',
    value: 'Planned',
    icon: statusIcon.Todo,
    iconClass: statusColor.Todo,
  },
  {
    label: 'In Progress',
    value: 'In Progress',
    icon: statusIcon['In Progress'],
    iconClass: statusColor['In Progress'],
  },
  { label: 'Completed', value: 'Completed', icon: statusIcon.Done, iconClass: statusColor.Done },
  {
    label: 'Canceled',
    value: 'Canceled',
    icon: statusIcon.Canceled,
    iconClass: statusColor.Canceled,
  },
] as const;

export const projectPriorities = priorities.map((item) => ({
  label: item,
  value: item,
  icon: 'i-tabler-antenna-bars-5',
  iconClass: priorityColor[item],
}));

export function projectPickerItems(projects: ProjectRecord[]) {
  return [
    ...projects.map((project) => ({
      label: project.name,
      value: project.name,
      icon: project.icon || defaultProjectIcon,
      iconClass: iconColor(project.color).text,
    })),
    {
      label: 'Unassigned',
      value: 'Unassigned',
      icon: defaultProjectIcon,
      iconClass: 'text-foreground-muted',
    },
  ];
}
