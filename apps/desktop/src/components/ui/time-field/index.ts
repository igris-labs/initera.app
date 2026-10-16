import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const timeFieldDefaults = {
  size: uiConfig.components?.timeField?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.timeField?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const timeFieldVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: timeFieldDefaults.variant,
    size: timeFieldDefaults.size,
  },
});
export type TimeFieldVariants = VariantProps<typeof timeFieldVariants>;
export { default as TimeField } from './TimeField.vue';

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center gap-1 rounded-lg border border-outline bg-default text-foreground',
  segment: 'rounded-sm px-0.5 focus:bg-accented focus:outline-none',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'min-h-7 gap-1.5 px-2.5 text-xs',
  },
  md: {
    root: 'min-h-8 gap-1.5 px-3 text-sm',
  },
  lg: {
    root: 'min-h-9 gap-2 px-3.5 text-base',
  },
};

export function timeFieldUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? timeFieldDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, timeFieldVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
