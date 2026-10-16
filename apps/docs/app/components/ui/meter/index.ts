import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Meter } from './Meter.vue';
export const meterDefaults = {
  size: uiConfig.components?.meter?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.meter?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const meterVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: meterDefaults.variant,
    size: meterDefaults.size,
  },
});
export type MeterVariants = VariantProps<typeof meterVariants>;

export const uiParts: Record<string, string> = {
  root: 'grid gap-1.5 text-sm',
  track: 'h-2 overflow-hidden rounded-full bg-muted',
  indicator: 'h-full rounded-full bg-inverted',
  label: 'font-medium',
  value: 'text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    track: 'h-1.5',
    label: 'text-xs',
    value: 'text-xs',
  },
  md: {
    track: 'h-2',
    label: 'text-sm',
    value: 'text-xs',
  },
  lg: {
    track: 'h-3',
    label: 'text-base',
    value: 'text-sm',
  },
};

export function meterUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? meterDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, meterVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
