import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Calendar } from './Calendar.vue';
export const calendarDefaults = {
  size: uiConfig.components?.calendar?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.calendar?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const calendarVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: calendarDefaults.variant,
    size: calendarDefaults.size,
  },
});
export type CalendarVariants = VariantProps<typeof calendarVariants>;

export const uiParts: Record<string, string> = {
  root: 'w-max rounded-lg border border-outline bg-elevated p-3 text-foreground shadow-lg',
  header: 'mb-2 flex items-center justify-between gap-2',
  previous:
    'inline-flex size-7 items-center justify-center rounded-md text-foreground-muted transition-colors duration-150 hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-inverted disabled:opacity-40',
  next: 'inline-flex size-7 items-center justify-center rounded-md text-foreground-muted transition-colors duration-150 hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-inverted disabled:opacity-40',
  heading: 'text-xs font-semibold text-foreground',
  grids: 'flex gap-3',
  grid: 'border-collapse',
  gridHead: 'border-b border-outline',
  gridRow: 'h-8',
  headCell: 'h-7 w-8 text-center text-xs font-medium text-foreground-muted',
  cell: 'p-0 text-center',
  cellTrigger:
    'inline-flex items-center justify-center rounded-md text-xs text-foreground transition-colors duration-100 hover:bg-accented focus-visible:outline-2 focus-visible:outline-inverted data-akaza-today:font-bold data-akaza-outside-view:text-foreground-dimmed data-akaza-selected:bg-inverted data-akaza-selected:text-foreground-inverted data-akaza-disabled:opacity-35 data-akaza-unavailable:opacity-35',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    cellTrigger: 'size-7 text-xs',
    previous: 'size-7 text-xs',
    next: 'size-7 text-xs',
    heading: 'text-xs',
    headCell: 'text-xs',
  },
  md: {
    cellTrigger: 'size-8 text-sm',
    previous: 'size-8 text-sm',
    next: 'size-8 text-sm',
    heading: 'text-sm',
    headCell: 'text-sm',
  },
  lg: {
    cellTrigger: 'size-9 text-base',
    previous: 'size-9 text-base',
    next: 'size-9 text-base',
    heading: 'text-base',
    headCell: 'text-base',
  },
};

export function calendarUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? calendarDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, calendarVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
