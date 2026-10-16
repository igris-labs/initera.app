import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Form } from './Form.vue';
export const formDefaults = {
  size: uiConfig.components?.form?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.form?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const formVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: formDefaults.variant,
    size: formDefaults.size,
  },
});
export type FormVariants = VariantProps<typeof formVariants>;

export const uiParts: Record<string, string> = { root: 'grid gap-4' };

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function formUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? formDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, formVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
