import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as DateRangePicker } from './DateRangePicker.vue';
export const dateRangePickerDefaults = {
  size: uiConfig.components?.dateRangePicker?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.dateRangePicker?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const dateRangePickerVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: dateRangePickerDefaults.variant,
    size: dateRangePickerDefaults.size,
  },
});
export type DateRangePickerVariants = VariantProps<typeof dateRangePickerVariants>;

export const uiParts: Record<string, string> = {
  trigger:
    'inline-flex cursor-pointer items-center rounded-lg border border-outline bg-default text-foreground transition-colors duration-150 hover:bg-muted focus-visible:outline-2 focus-visible:outline-inverted',
  content: 'z-50 rounded-lg border border-outline bg-elevated p-3 shadow-lg',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    trigger: 'min-h-7 gap-1.5 px-2.5 text-xs',
  },
  md: {
    trigger: 'min-h-8 gap-1.5 px-3 text-sm',
  },
  lg: {
    trigger: 'min-h-9 gap-2 px-3.5 text-base',
  },
};

export function dateRangePickerUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? dateRangePickerDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    dateRangePickerVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
