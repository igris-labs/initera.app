import type { Ref } from 'vue';
import type { Router } from 'vue-router';
import type { Command } from '../../lib/command-search';
import { searchableDescription } from '../../lib/rich-text-search';
import { usesNativeStorage } from '../../lib/workspace-storage';
import type { ProjectRecord } from '../../lib/workspace-storage';
import type { PlanningNodeSearchResult } from '../../lib/issue-storage';
import type { CommentSearchResult, Issue } from '../../types/issue';
import type { useIssuesStore } from '../../stores/issues';
import type { useProjectsStore } from '../../stores/projects';
import type { useCyclesStore } from '../../stores/cycles';
import type { useViewsStore } from '../../stores/views';
import type { useWorkspaceStore } from '../../stores/workspace';

interface RecordCommandContext {
  router: Router;
  issues: ReturnType<typeof useIssuesStore>;
  projects: ReturnType<typeof useProjectsStore>;
  cycles: ReturnType<typeof useCyclesStore>;
  views: ReturnType<typeof useViewsStore>;
  workspace: ReturnType<typeof useWorkspaceStore>;
  nativeIssueRecords: Ref<Issue[]>;
  nativeProjectRecords: Ref<ProjectRecord[]>;
  nativePlanningMatches: Ref<PlanningNodeSearchResult[]>;
}

function sidebarAction(name: string, detail?: unknown) {
  window.dispatchEvent(new CustomEvent(name, { detail }));
}

export function commentCommand(router: Router, comment: CommentSearchResult): Command {
  return {
    id: `comment-${comment.activityId}`,
    label: comment.body.replace(/\s+/g, ' ').slice(0, 180),
    detail: comment.identifier,
    keywords: comment.body,
    section: 'Comments',
    icon: 'i-tabler-message-circle',
    run: () =>
      router.push({
        name: 'issue-full',
        params: { id: comment.identifier },
        query: { comment: comment.activityId },
      }),
  };
}

export function recordCommands({
  router,
  issues,
  projects,
  cycles,
  views,
  workspace,
  nativeIssueRecords,
  nativeProjectRecords,
  nativePlanningMatches,
}: RecordCommandContext) {
  const aliasesByCurrent = new Map<string, string[]>();
  for (const [alias, current] of Object.entries(issues.identifierAliases)) {
    aliasesByCurrent.set(current, [...(aliasesByCurrent.get(current) ?? []), alias]);
  }

  const availableIssues = new Map(
    [...issues.items, ...nativeIssueRecords.value].map((issue) => [issue.id, issue]),
  );
  const issueResults: Command[] = [...availableIssues.values()].map((issue) => ({
    id: `issue-${issue.id}`,
    label: issue.title,
    detail: issue.id,
    keywords: `${searchableDescription(issue.descriptionDocument, issue.description)} ${issue.project} ${issue.labels.join(' ')} ${(aliasesByCurrent.get(issue.id) ?? []).join(' ')}`,
    section: 'Issues',
    icon: 'i-tabler-circle-dashed',
    run: () => router.push({ name: 'issue-full', params: { id: issue.id } }),
  }));
  const availableProjects = new Map(
    [...projects.items, ...nativeProjectRecords.value].map((project) => [project.id, project]),
  );
  const projectResults: Command[] = [...availableProjects.values()].map((project) => ({
    id: `project-${project.id}`,
    label: project.name,
    detail: project.identifierPrefix,
    keywords: `${project.summary} ${searchableDescription(project.descriptionDocument, project.description)} ${(project.labels ?? []).join(' ')}`,
    section: 'Projects',
    icon: project.icon || 'i-tabler-cube',
    run: () => router.push({ name: 'project-overview', params: { project: project.name } }),
  }));
  const planningResults: Command[] = nativePlanningMatches.value.map((node) => ({
    id: `node-${node.id}`,
    label: node.name,
    detail: `${node.projectName} · ${node.kind}`,
    section: 'Project structure',
    icon: node.kind === 'module' ? 'i-tabler-folders' : 'i-tabler-diamond',
    run: () =>
      router.push({
        name: 'project-structure',
        params: { project: node.projectName },
        query: { node: node.id },
      }),
  }));
  const previewComments: Command[] = usesNativeStorage
    ? []
    : issues.items.flatMap((issue) =>
        (issues.activityByIssue[issue.id] ?? [])
          .filter((entry) => entry.kind === 'comment')
          .map((entry) =>
            commentCommand(router, {
              activityId: entry.id,
              identifier: issue.id,
              body: entry.body,
            }),
          ),
      );
  const viewResults: Command[] = views.items.map((view) => ({
    id: `view-${view.id}`,
    label: view.name,
    section: 'Views',
    icon: 'i-tabler-stack-2',
    run: () => router.push({ name: 'saved-view', params: { viewId: view.id } }),
  }));
  const cycleResults: Command[] = cycles.items.map((cycle) => ({
    id: `cycle-${cycle.id}`,
    label: cycle.title,
    detail: `${cycle.startDate} – ${cycle.endDate}`,
    section: 'Cycles',
    icon: 'i-tabler-calendar-time',
    run: () => router.push('/cycles'),
  }));
  const workspaceResults: Command[] = workspace.workspaces
    .filter((item) => item.id !== workspace.workspaceId)
    .map((item) => ({
      id: `workspace-${item.id}`,
      label: `Switch to ${item.name}`,
      section: 'Workspaces',
      icon: 'i-tabler-switch-horizontal',
      run: () => sidebarAction('workspace-command', { action: 'switch', id: item.id }),
    }));
  return {
    issueResults,
    projectResults,
    planningResults,
    previewComments,
    viewResults,
    cycleResults,
    workspaceResults,
  };
}
