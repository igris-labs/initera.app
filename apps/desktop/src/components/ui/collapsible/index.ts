import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Collapsible } from './Collapsible.vue';
export const collapsibleDefaults = {
  size: uiConfig.components?.collapsible?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.collapsible?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const collapsibleVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: collapsibleDefaults.variant,
    size: collapsibleDefaults.size,
  },
});
export type CollapsibleVariants = VariantProps<typeof collapsibleVariants>;

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  trigger: 'flex min-h-7 items-center gap-2 rounded-md px-2 hover:bg-muted',
  content: 'p-2',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function collapsibleUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? collapsibleDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, collapsibleVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
