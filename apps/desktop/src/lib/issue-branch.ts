import type { Issue } from '../types/issue';
import { notifications } from './notifications';

export function issueBranchName(issue: Pick<Issue, 'id' | 'title'>) {
  const slug =
    issue.title
      .normalize('NFKD')
      .replace(/\p{Mark}/gu, '')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 80)
      .replace(/-+$/g, '') || 'issue';

  return `${issue.id}-${slug}`;
}

export async function copyIssueBranchName(issue: Pick<Issue, 'id' | 'title'>) {
  const branch = issueBranchName(issue);
  try {
    await navigator.clipboard.writeText(branch);
    notifications.add({ title: 'Branch name copied', description: branch, type: 'success' });
  } catch {
    notifications.add({ title: 'Could not copy branch name', description: branch, type: 'error' });
  }
}
