export const defaultProjectIcon = 'i-tabler-cube';
export const defaultProjectColor = 'amber';

export const iconChoices = [
  { name: 'i-tabler-cube', label: 'Cube', group: 'Objects' },
  { name: 'i-tabler-stack-2', label: 'Layers', group: 'Objects' },
  { name: 'i-tabler-device-desktop', label: 'Desktop', group: 'Objects' },
  { name: 'i-tabler-palette', label: 'Palette', group: 'Objects' },
  { name: 'i-tabler-bookmark', label: 'Bookmark', group: 'Objects' },
  { name: 'i-tabler-inbox', label: 'Inbox', group: 'Objects' },
  { name: 'i-tabler-tag', label: 'Tag', group: 'Objects' },
  { name: 'i-tabler-target', label: 'Target', group: 'Planning' },
  { name: 'i-tabler-flag', label: 'Flag', group: 'Planning' },
  { name: 'i-tabler-calendar', label: 'Calendar', group: 'Planning' },
  { name: 'i-tabler-clock', label: 'Clock', group: 'Planning' },
  { name: 'i-tabler-rocket', label: 'Rocket', group: 'Planning' },
  { name: 'i-tabler-bulb', label: 'Idea', group: 'Planning' },
  { name: 'i-tabler-star', label: 'Star', group: 'Planning' },
  { name: 'i-tabler-code', label: 'Code', group: 'Work' },
  { name: 'i-tabler-bug', label: 'Bug', group: 'Work' },
  { name: 'i-tabler-git-branch', label: 'Branch', group: 'Work' },
  { name: 'i-tabler-git-merge', label: 'Merge', group: 'Work' },
  { name: 'i-tabler-shield-check', label: 'Shield', group: 'Work' },
  { name: 'i-tabler-sparkles', label: 'Sparkles', group: 'Work' },
  { name: 'i-tabler-bolt', label: 'Bolt', group: 'Work' },
  { name: 'i-tabler-flame', label: 'Flame', group: 'Work' },
  { name: 'i-tabler-message-circle', label: 'Message', group: 'Work' },
  { name: 'i-tabler-list', label: 'List', group: 'Work' },
] as const;

export const iconColors = {
  gray: { label: 'Gray', swatch: 'var(--label-gray)', text: 'text-(--label-gray)' },
  blue: { label: 'Blue', swatch: 'var(--label-blue)', text: 'text-(--label-blue)' },
  purple: { label: 'Purple', swatch: 'var(--label-purple)', text: 'text-(--label-purple)' },
  green: { label: 'Green', swatch: 'var(--label-green)', text: 'text-(--label-green)' },
  amber: { label: 'Amber', swatch: 'var(--label-amber)', text: 'text-(--label-amber)' },
  red: { label: 'Red', swatch: 'var(--label-red)', text: 'text-(--label-red)' },
  cyan: { label: 'Cyan', swatch: 'var(--label-cyan)', text: 'text-(--label-cyan)' },
  peach: { label: 'Peach', swatch: 'var(--label-peach)', text: 'text-(--label-peach)' },
} as const;

export type IconColor = keyof typeof iconColors;

export function iconColor(color: string | undefined) {
  return iconColors[color as IconColor] ?? iconColors[defaultProjectColor];
}
