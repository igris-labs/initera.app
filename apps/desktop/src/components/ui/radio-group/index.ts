import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as RadioGroup } from './RadioGroup.vue';
export const radioGroupDefaults = {
  size: uiConfig.components?.radioGroup?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.radioGroup?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const radioGroupVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: radioGroupDefaults.variant,
    size: radioGroupDefaults.size,
  },
});
export type RadioGroupVariants = VariantProps<typeof radioGroupVariants>;

export const uiParts: Record<string, string> = {
  root: 'grid gap-2 text-sm',
  item: 'flex items-center gap-2',
  indicator: 'size-4 rounded-full border border-outline data-[akaza-state=checked]:border-inverted',
  label: 'font-medium',
  description: 'text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    label: 'text-xs',
    description: 'text-xs',
  },
  md: {
    label: 'text-sm',
    description: 'text-xs',
  },
  lg: {
    label: 'text-base',
    description: 'text-sm',
  },
};

export function radioGroupUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? radioGroupDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, radioGroupVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
