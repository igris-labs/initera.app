import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const toggleDefaults = {
  size: uiConfig.components?.toggle?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.toggle?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const toggleVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: toggleDefaults.variant,
    size: toggleDefaults.size,
  },
});
export type ToggleVariants = VariantProps<typeof toggleVariants>;
export { default as Toggle } from './Toggle.vue';

export const uiParts: Record<string, string> = {
  root: 'inline-flex min-h-7 items-center justify-center rounded-lg border border-transparent px-2.5 text-xs font-medium hover:bg-muted data-[akaza-state=on]:border-outline data-[akaza-state=on]:bg-accented',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'min-h-7 gap-1.5 px-2.5 text-xs',
  },
  md: {
    root: 'min-h-8 gap-1.5 px-3 text-sm',
  },
  lg: {
    root: 'min-h-9 gap-2 px-3.5 text-base',
  },
};

export function toggleUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? toggleDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, toggleVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
