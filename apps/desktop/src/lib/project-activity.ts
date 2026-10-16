import type { ProjectInput, ProjectRecord } from './workspace-storage';

export function projectChanges(previous: ProjectRecord, next: ProjectInput): string {
  const changes: string[] = [];

  if (previous.name !== next.name)
    changes.push(`renamed project from ${previous.name} to ${next.name}`);
  if (previous.nameMarkdown !== next.nameMarkdown)
    changes.push('edited the project title formatting');
  if (previous.identifierPrefix !== next.identifierPrefix)
    changes.push('changed the project identifier');
  if (previous.status !== next.status)
    changes.push(`changed status from ${previous.status} to ${next.status}`);
  if (previous.priority !== next.priority)
    changes.push(`changed priority from ${previous.priority} to ${next.priority}`);
  if ((previous.leadActorId ?? null) !== next.leadActorId)
    changes.push(next.leadActorId ? 'assigned the project lead' : 'removed the project lead');
  if (previous.startDate !== next.startDate) changes.push('changed the start date');
  if (previous.targetDate !== next.targetDate) changes.push('changed the target date');
  if (JSON.stringify(previous.labels) !== JSON.stringify(next.labels))
    changes.push('changed labels');
  if (previous.icon !== next.icon || previous.color !== next.color)
    changes.push('changed the project icon or color');
  if (previous.summary !== next.summary) changes.push('edited the summary');
  if (
    previous.description !== next.description ||
    JSON.stringify(previous.descriptionDocument) !== JSON.stringify(next.descriptionDocument)
  )
    changes.push('edited the description');
  if (JSON.stringify(previous.resources) !== JSON.stringify(next.resources))
    changes.push('changed resources');
  if (JSON.stringify(previous.milestones) !== JSON.stringify(next.milestones))
    changes.push('changed milestones');

  return changes.join('; ');
}
