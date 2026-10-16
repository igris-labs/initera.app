import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Rating } from './Rating.vue';
export const ratingDefaults = {
  size: uiConfig.components?.rating?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.rating?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const ratingVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: ratingDefaults.variant,
    size: ratingDefaults.size,
  },
});
export type RatingVariants = VariantProps<typeof ratingVariants>;

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center gap-1',
  item: 'size-5 text-foreground-dimmed data-[akaza-state=active]:text-warning',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    item: 'size-4',
  },
  md: {
    item: 'size-5',
  },
  lg: {
    item: 'size-6',
  },
};

export function ratingUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? ratingDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, ratingVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
