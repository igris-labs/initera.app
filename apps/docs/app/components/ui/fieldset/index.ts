import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Fieldset } from './Fieldset.vue';
export const fieldsetDefaults = {
  size: uiConfig.components?.fieldset?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.fieldset?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const fieldsetVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: fieldsetDefaults.variant,
    size: fieldsetDefaults.size,
  },
});
export type FieldsetVariants = VariantProps<typeof fieldsetVariants>;

export const uiParts: Record<string, string> = {
  root: 'grid gap-3 rounded-lg border border-outline p-3',
  legend: 'px-1 text-sm font-medium',
  description: 'text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    legend: 'text-xs',
    description: 'text-xs',
  },
  md: {
    legend: 'text-sm',
    description: 'text-xs',
  },
  lg: {
    legend: 'text-base',
    description: 'text-sm',
  },
};

export function fieldsetUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? fieldsetDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, fieldsetVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
