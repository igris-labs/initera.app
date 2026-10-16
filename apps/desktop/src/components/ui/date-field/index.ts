import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as DateField } from './DateField.vue';
export const dateFieldDefaults = {
  size: uiConfig.components?.dateField?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.dateField?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const dateFieldVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: dateFieldDefaults.variant,
    size: dateFieldDefaults.size,
  },
});
export type DateFieldVariants = VariantProps<typeof dateFieldVariants>;

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center gap-1 rounded-lg border border-outline bg-default text-foreground',
  segments: 'inline-flex items-center',
  segment: 'rounded-sm px-0.5 focus:bg-accented focus:outline-none',
  calendarTrigger:
    'inline-flex items-center gap-1.5 rounded-md text-xs text-foreground-muted transition-colors duration-100 hover:bg-accented hover:text-foreground focus-visible:outline-2 focus-visible:outline-inverted',
  calendarContent: 'z-50 rounded-lg',
  hiddenInput: 'sr-only',
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

export function dateFieldUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? dateFieldDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, dateFieldVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
