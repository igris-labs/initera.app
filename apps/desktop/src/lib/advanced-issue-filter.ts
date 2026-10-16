import type { Issue, IssueFilterGroup, IssueFilterRule, IssueRelation } from '../types/issue';

function dateValue(value: string) {
  if (/^\d{4}-\d{2}-\d{2}/.test(value)) return value.slice(0, 10);

  const today = new Date();
  if (value === 'Yesterday') today.setDate(today.getDate() - 1);
  if (value === 'Today' || value === 'Yesterday') return today.toISOString().slice(0, 10);

  const parsed = new Date(`${value} ${today.getFullYear()}`);
  return Number.isNaN(parsed.getTime()) ? '' : parsed.toISOString().slice(0, 10);
}

function matchesRule(
  issue: Issue,
  rule: IssueFilterRule,
  issues: Issue[],
  relations: IssueRelation[],
) {
  let value: string | undefined;

  switch (rule.field) {
    case 'due':
      value = issue.due;
      break;
    case 'start':
      value = issue.startDate ?? undefined;
      break;
    case 'created':
      value = issue.created ?? issue.updated;
      break;
    case 'updated':
      value = issue.updated;
      break;
    case 'triaged':
      value = issue.triagedAt ?? undefined;
      break;
    case 'parent':
      value = issue.parentId ?? undefined;
      break;
    case 'subissues':
      value = issues.some((item) => item.parentId === issue.id && !item.archivedAt)
        ? 'yes'
        : undefined;
      break;
    case 'blocks':
      value = relations.some((item) => item.kind === 'blocks' && item.sourceId === issue.id)
        ? 'yes'
        : undefined;
      break;
    case 'blockedBy':
      value = relations.some((item) => item.kind === 'blocks' && item.targetId === issue.id)
        ? 'yes'
        : undefined;
      break;
    case 'related':
      value = relations.some(
        (item) =>
          item.kind === 'related' && (item.sourceId === issue.id || item.targetId === issue.id),
      )
        ? 'yes'
        : undefined;
      break;
    case 'duplicates':
      value = relations.some(
        (item) =>
          item.kind === 'duplicates' && (item.sourceId === issue.id || item.targetId === issue.id),
      )
        ? 'yes'
        : undefined;
      break;
  }

  if (rule.operator === 'has') return Boolean(value);
  if (rule.operator === 'none') return !value;
  if (!value || !rule.value) return false;

  const date = dateValue(value);
  if (!date) return false;
  if (rule.operator === 'before') return date < rule.value;
  if (rule.operator === 'after') return date > rule.value;
  return Boolean(rule.end && date >= rule.value && date <= rule.end);
}

export function matchesAdvancedFilters(
  issue: Issue,
  groups: IssueFilterGroup[],
  issues: Issue[],
  relations: IssueRelation[],
) {
  return groups.every((group) => {
    if (!group.rules.length) return false;
    const matches = group.rules.map((rule) => matchesRule(issue, rule, issues, relations));
    return group.mode === 'all' ? matches.every(Boolean) : matches.some(Boolean);
  });
}
