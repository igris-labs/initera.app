import { reactive } from 'vue';

export const labelColor: Record<string, string> = reactive({
  UI: 'var(--label-indigo)',
  Feature: 'var(--label-purple)',
  Core: 'var(--label-red)',
  Bug: 'var(--label-red)',
  Interaction: 'var(--label-green)',
  Feedback: 'var(--label-green)',
  Workflow: 'var(--label-amber)',
  Improvement: 'var(--label-blue)',
  Keyboard: 'var(--label-gray)',
  Accessibility: 'var(--label-blue)',
  Infrastructure: 'var(--label-cyan)',
  Design: 'var(--label-peach)',
  Security: 'var(--label-gray)',
});

export function labelSwatch(label: string): string {
  return labelColor[label] ?? 'var(--label-gray)';
}
