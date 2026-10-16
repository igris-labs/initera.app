export type IssueDetailAction =
  | 'edit'
  | 'archive'
  | 'subissue'
  | 'parent'
  | 'status'
  | 'priority'
  | 'estimate'
  | 'relation'
  | 'project'
  | 'labels';

export function dispatchIssueDetailAction(issueId: string, action: IssueDetailAction) {
  window.dispatchEvent(new CustomEvent('issue-detail-command', { detail: { issueId, action } }));
}
