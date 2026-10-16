import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const timeRangeFieldDefaults = {
  size: uiConfig.components?.timeRangeField?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.timeRangeField?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const timeRangeFieldVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: timeRangeFieldDefaults.variant,
    size: timeRangeFieldDefaults.size,
  },
});
export type TimeRangeFieldVariants = VariantProps<typeof timeRangeFieldVariants>;
export { default as TimeRangeField } from './TimeRangeField.vue';

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center gap-1 rounded-lg border border-outline bg-default text-foreground',
  separator: 'text-foreground-muted',
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

export function timeRangeFieldUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? timeRangeFieldDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(
    styles.root,
    timeRangeFieldVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
