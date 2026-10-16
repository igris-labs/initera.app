import type { Ref } from 'vue';
import type { Router } from 'vue-router';
import type { Command } from '../../lib/command-search';
import { dispatchIssueDetailAction } from '../../lib/issue-detail-command';
import type { IssueDetailAction } from '../../lib/issue-detail-command';
import type { Issue } from '../../types/issue';
import type { useIssuesStore } from '../../stores/issues';

interface ContextCommandContext {
  router: Router;
  issues: ReturnType<typeof useIssuesStore>;
  projectName: Ref<string>;
  issueName: Ref<string>;
  currentIssue: Ref<Issue | undefined>;
}

export function contextCommands({
  router,
  issues,
  projectName,
  issueName,
  currentIssue,
}: ContextCommandContext): Command[] {
  const actions: Command[] = [];

  function projectAction(name: string) {
    window.dispatchEvent(new CustomEvent('project-command', { detail: name }));
  }

  function issueDetailAction(action: IssueDetailAction) {
    dispatchIssueDetailAction(issueName.value, action);
  }

  if (projectName.value)
    actions.unshift(
      {
        id: 'project-icon',
        label: 'Change project icon and color',
        section: projectName.value,
        icon: 'i-tabler-palette',
        shortcut: 'projectIcon',
        run: () =>
          document
            .querySelector<HTMLElement>('[aria-label="Choose project icon and color"]')
            ?.click(),
      },
      {
        id: 'project-status',
        label: 'Change project status',
        section: projectName.value,
        icon: 'i-tabler-circle-dashed',
        shortcut: 'projectStatus',
        run: () => projectAction('status'),
      },
      {
        id: 'project-priority',
        label: 'Change project priority',
        section: projectName.value,
        icon: 'i-tabler-antenna-bars-5',
        shortcut: 'projectPriority',
        run: () => projectAction('priority'),
      },
      {
        id: 'project-lead',
        label: 'Set project lead',
        section: projectName.value,
        icon: 'i-tabler-user',
        shortcut: 'projectLead',
        run: () => projectAction('lead'),
      },
      {
        id: 'project-labels',
        label: 'Add project labels',
        section: projectName.value,
        icon: 'i-tabler-tag',
        shortcut: 'projectLabels',
        run: () => projectAction('labels'),
      },
      {
        id: 'project-target-date',
        label: 'Set project target date',
        section: projectName.value,
        icon: 'i-tabler-calendar',
        shortcut: 'projectTargetDate',
        run: () => projectAction('targetDate'),
      },
      {
        id: 'project-start-date',
        label: 'Set project start date',
        section: projectName.value,
        icon: 'i-tabler-calendar',
        shortcut: 'projectStartDate',
        run: () => projectAction('startDate'),
      },
      {
        id: 'project-rename',
        label: 'Rename project',
        section: projectName.value,
        icon: 'i-tabler-pencil',
        shortcut: 'projectRename',
        run: () => projectAction('rename'),
      },
      {
        id: 'project-update',
        label: 'Write project update',
        section: projectName.value,
        icon: 'i-tabler-pencil-plus',
        shortcut: 'projectUpdate',
        run: () => projectAction('update'),
      },
      {
        id: 'project-milestone',
        label: 'Add milestone',
        section: projectName.value,
        icon: 'i-tabler-flag',
        shortcut: 'projectMilestone',
        run: () => projectAction('milestone'),
      },
      {
        id: 'project-resource',
        label: 'Add document or link',
        section: projectName.value,
        icon: 'i-tabler-plus',
        shortcut: 'projectResource',
        run: () => projectAction('resource'),
      },
      {
        id: 'project-issues',
        label: 'View project issues',
        section: projectName.value,
        icon: 'i-tabler-list',
        shortcut: 'projectIssues',
        run: () => router.push({ name: 'project-issues', params: { project: projectName.value } }),
      },
    );
  if (issueName.value)
    actions.unshift(
      ...(currentIssue.value?.triage
        ? [
            {
              id: 'issue-accept-triage',
              label: 'Accept from triage',
              section: issueName.value,
              icon: 'i-tabler-check',
              run: () => {
                issues.update(issueName.value, {
                  triage: false,
                  status:
                    currentIssue.value &&
                    issues.categoryFor(currentIssue.value.status) === 'Backlog'
                      ? 'Todo'
                      : currentIssue.value?.status,
                });
                return router.push('/issues');
              },
            },
            {
              id: 'issue-decline-triage',
              label: 'Decline from triage',
              section: issueName.value,
              icon: 'i-tabler-x',
              run: () => {
                issues.update(issueName.value, { triage: false, status: 'Canceled' });
                return router.push('/issues');
              },
            },
          ]
        : currentIssue.value?.project === 'Unassigned'
          ? [
              {
                id: 'issue-send-to-triage',
                label: 'Send to triage',
                section: issueName.value,
                icon: 'i-tabler-inbox',
                run: () => {
                  issues.update(issueName.value, { triage: true, status: 'Backlog' });
                  return router.push('/triage');
                },
              },
            ]
          : []),
      {
        id: 'issue-edit',
        label: 'Edit issue title and description',
        section: issueName.value,
        icon: 'i-tabler-pencil',
        shortcut: 'issueEdit',
        run: () => issueDetailAction('edit'),
      },
      {
        id: 'issue-archive',
        label: 'Archive issue',
        section: issueName.value,
        icon: 'i-tabler-archive',
        shortcut: 'issueArchive',
        run: () => issueDetailAction('archive'),
      },
      {
        id: 'issue-subissue',
        label: 'Add sub-issue',
        section: issueName.value,
        icon: 'i-tabler-list-details',
        shortcut: 'createSubissue',
        run: () => issueDetailAction('subissue'),
      },
      {
        id: 'issue-parent',
        label: 'Set parent issue',
        section: issueName.value,
        icon: 'i-tabler-hierarchy',
        shortcut: 'issueParent',
        run: () => issueDetailAction('parent'),
      },
      {
        id: 'issue-status',
        label: 'Change issue status',
        section: issueName.value,
        icon: 'i-tabler-circle-dashed',
        shortcut: 'issueStatus',
        run: () => issueDetailAction('status'),
      },
      {
        id: 'issue-priority',
        label: 'Change issue priority',
        section: issueName.value,
        icon: 'i-tabler-antenna-bars-5',
        shortcut: 'issuePriority',
        run: () => issueDetailAction('priority'),
      },
      {
        id: 'issue-estimate',
        label: 'Set issue estimate',
        section: issueName.value,
        icon: 'i-tabler-hash',
        shortcut: 'issueEstimate',
        run: () => issueDetailAction('estimate'),
      },
      {
        id: 'issue-relation',
        label: 'Link another issue',
        section: issueName.value,
        icon: 'i-tabler-link',
        shortcut: 'issueRelation',
        run: () => issueDetailAction('relation'),
      },
      {
        id: 'issue-project',
        label: 'Move issue to project',
        section: issueName.value,
        icon: 'i-tabler-cube',
        shortcut: 'issueProject',
        run: () => issueDetailAction('project'),
      },
      {
        id: 'issue-labels',
        label: 'Add issue labels',
        section: issueName.value,
        icon: 'i-tabler-tag',
        shortcut: 'issueLabels',
        run: () => issueDetailAction('labels'),
      },
    );
  return actions;
}
