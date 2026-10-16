import type { ProjectRecord } from '../../src/lib/workspace-storage';

export const previewProjects: ProjectRecord[] = ['Desktop app', 'Design system'].map(
  (name, index) => ({
    id: `preview-${index}`,
    workspaceId: 'preview',
    name,
    nameMarkdown: '',
    identifierPrefix: name === 'Desktop app' ? 'APP' : 'DSN',
    icon: 'i-tabler-cube',
    color: 'amber',
    createdAt: '',
    updatedAt: '',
    summary: '',
    description: '',
    status: 'Backlog',
    priority: 'No priority',
    leadActorId: null,
    startDate: null,
    targetDate: null,
    labels: [],
    resources: [],
    milestones: [],
  }),
);
