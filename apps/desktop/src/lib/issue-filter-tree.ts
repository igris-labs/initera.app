import type { Issue, IssueFilterTree } from '../types/issue';

interface FilterContext {
  projectId?: string;
  nodeId?: string;
  cycleId?: string;
  statusCategory?: string;
  labelIds?: string[];
}

export function matchesIssueFilterTree(
  issue: Issue,
  node: IssueFilterTree | undefined,
  context: FilterContext = {},
): boolean {
  if (!node) return true;
  if ('children' in node) {
    if (!node.children.length) return true;

    return node.op === 'and'
      ? node.children.every((child) => matchesIssueFilterTree(issue, child, context))
      : node.children.some((child) => matchesIssueFilterTree(issue, child, context));
  }

  const field = node.field;
  if (field === 'labelIds') {
    const selected = Array.isArray(node.value) ? node.value : [];
    const assigned = context.labelIds ?? [];
    if (node.operator === 'hasAny') return selected.some((id) => assigned.includes(String(id)));
    if (node.operator === 'hasAll') return selected.every((id) => assigned.includes(String(id)));
    if (node.operator === 'hasNone') return selected.every((id) => !assigned.includes(String(id)));
    return false;
  }

  const actual: string | number | null =
    {
      projectId: context.projectId,
      nodeId: context.nodeId,
      milestoneId: issue.milestoneId,
      cycleId: context.cycleId,
      statusId: issue.status,
      statusCategory: context.statusCategory,
      priority: issue.priority,
      intakeState: issue.triage ? 1 : 0,
      title: issue.title,
      currentIdentifier: issue.id,
      estimatePoints: issue.estimate,
      dueDate: issue.due,
      createdAt: issue.created,
      updatedAt: issue.updated,
      parentIssueId: issue.parentId,
    }[field] ?? null;
  const value = node.value;

  switch (node.operator) {
    case 'isSet':
      return actual !== null;
    case 'isUnset':
      return actual === null;
    case 'eq':
      return actual !== null && actual === value;
    case 'equals':
      return (
        typeof actual === 'string' &&
        typeof value === 'string' &&
        actual.toLowerCase() === value.toLowerCase()
      );
    case 'neq':
      return actual !== null && actual !== value;
    case 'in':
      return actual !== null && Array.isArray(value) && value.includes(actual);
    case 'notIn':
      return actual !== null && Array.isArray(value) && !value.includes(actual);
    case 'contains':
      return (
        typeof actual === 'string' &&
        typeof value === 'string' &&
        actual.toLowerCase().includes(value.toLowerCase())
      );
    case 'gt':
      return typeof actual === 'number' && typeof value === 'number' && actual > value;
    case 'gte':
      return typeof actual === 'number' && typeof value === 'number' && actual >= value;
    case 'lt':
      return typeof actual === 'number' && typeof value === 'number' && actual < value;
    case 'lte':
      return typeof actual === 'number' && typeof value === 'number' && actual <= value;
    case 'before':
      return typeof actual === 'string' && typeof value === 'string' && actual.slice(0, 10) < value;
    case 'after':
      return typeof actual === 'string' && typeof value === 'string' && actual.slice(0, 10) > value;
    case 'on':
      return (
        typeof actual === 'string' && typeof value === 'string' && actual.slice(0, 10) === value
      );
    case 'between':
      return (
        typeof actual === 'string' &&
        Array.isArray(value) &&
        value.length === 2 &&
        actual.slice(0, 10) >= value[0]! &&
        actual.slice(0, 10) <= value[1]!
      );
    default:
      return false;
  }
}
