import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Progress } from './Progress.vue';
export const progressDefaults = {
  size: uiConfig.components?.progress?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.progress?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const progressVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: progressDefaults.variant,
    size: progressDefaults.size,
  },
});
export type ProgressVariants = VariantProps<typeof progressVariants>;

export const uiParts: Record<string, string> = {
  root: 'h-2 overflow-hidden rounded-full bg-muted',
  indicator: 'h-full bg-inverted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'h-1.5',
  },
  md: {
    root: 'h-2',
  },
  lg: {
    root: 'h-3',
  },
};

export function progressUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? progressDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, progressVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
