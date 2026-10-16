import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Checkbox } from './Checkbox.vue';
export const checkboxDefaults = {
  size: uiConfig.components?.checkbox?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.checkbox?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const checkboxVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: checkboxDefaults.variant,
    size: checkboxDefaults.size,
  },
});
export type CheckboxVariants = VariantProps<typeof checkboxVariants>;

export const uiParts: Record<string, string> = {
  wrapper: 'inline-flex items-start gap-2',
  root: 'size-4 shrink-0 rounded-sm border border-outline bg-default data-[akaza-state=checked]:border-inverted data-[akaza-state=checked]:bg-inverted focus-visible:outline-2 focus-visible:outline-inverted',
  indicator: 'flex items-center justify-center text-foreground-inverted',
  label: 'text-sm font-medium',
  description: 'text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'size-3.5',
    label: 'text-xs',
    description: 'text-xs',
  },
  md: {
    root: 'size-4',
    label: 'text-sm',
    description: 'text-xs',
  },
  lg: {
    root: 'size-5',
    label: 'text-base',
    description: 'text-sm',
  },
};

export function checkboxUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? checkboxDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, checkboxVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
