import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as CheckboxGroup } from './CheckboxGroup.vue';
export const checkboxGroupDefaults = {
  size: uiConfig.components?.checkboxGroup?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.checkboxGroup?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const checkboxGroupVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: checkboxGroupDefaults.variant,
    size: checkboxGroupDefaults.size,
  },
});
export type CheckboxGroupVariants = VariantProps<typeof checkboxGroupVariants>;

export const uiParts: Record<string, string> = {
  root: 'grid gap-2',
  legend: 'text-sm font-medium',
  item: 'flex items-center gap-2',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    legend: 'text-xs',
  },
  md: {
    legend: 'text-sm',
  },
  lg: {
    legend: 'text-base',
  },
};

export function checkboxGroupUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? checkboxGroupDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(
    styles.root,
    checkboxGroupVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
