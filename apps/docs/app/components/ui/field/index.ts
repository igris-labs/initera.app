import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Field } from './Field.vue';
export const fieldDefaults = {
  size: uiConfig.components?.field?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.field?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const fieldVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: fieldDefaults.variant,
    size: fieldDefaults.size,
  },
});
export type FieldVariants = VariantProps<typeof fieldVariants>;

export const uiParts: Record<string, string> = {
  root: 'grid gap-1.5',
  label: 'text-xs font-medium',
  description: 'text-xs text-foreground-muted',
  error: 'text-xs text-error',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    label: 'text-xs',
    description: 'text-xs',
    error: 'text-xs',
  },
  md: {
    label: 'text-xs',
    description: 'text-xs',
    error: 'text-xs',
  },
  lg: {
    label: 'text-sm',
    description: 'text-sm',
    error: 'text-sm',
  },
};

export function fieldUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? fieldDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, fieldVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
