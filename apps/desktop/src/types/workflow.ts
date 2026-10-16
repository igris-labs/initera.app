import type { StatusColor } from '../lib/issue-presentation';

export const workflowCategories = [
  'Backlog',
  'Unstarted',
  'Started',
  'Completed',
  'Canceled',
] as const;

export type WorkflowCategory = (typeof workflowCategories)[number];

export interface StatusAppearance {
  icon: string;
  color: StatusColor;
}

export interface StatusMeta {
  category: WorkflowCategory;
  description: string;
  isDefault: boolean;
}

export interface WorkflowStatus {
  name: string;
  category: WorkflowCategory;
  description: string;
  icon: string;
  color: StatusColor;
  position: number;
  archived: boolean;
  isDefault: boolean;
}
