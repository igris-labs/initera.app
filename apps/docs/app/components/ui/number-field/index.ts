import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as NumberField } from './NumberField.vue';
export const numberFieldDefaults = {
  size: uiConfig.components?.numberField?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.numberField?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const numberFieldVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: numberFieldDefaults.variant,
    size: numberFieldDefaults.size,
  },
});
export type NumberFieldVariants = VariantProps<typeof numberFieldVariants>;

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center overflow-hidden rounded-lg border border-outline bg-default',
  input: 'w-16 bg-transparent text-center text-sm outline-none',
  decrement: 'size-7 hover:bg-muted',
  increment: 'size-7 hover:bg-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'min-h-7 gap-1.5 px-2.5 text-xs',
    input: 'text-xs',
  },
  md: {
    root: 'min-h-8 gap-1.5 px-3 text-sm',
    input: 'text-sm',
  },
  lg: {
    root: 'min-h-9 gap-2 px-3.5 text-base',
    input: 'text-base',
  },
};

export function numberFieldUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? numberFieldDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, numberFieldVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
