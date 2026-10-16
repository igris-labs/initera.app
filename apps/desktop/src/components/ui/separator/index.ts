import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Separator } from './Separator.vue';
export const separatorDefaults = {
  size: uiConfig.components?.separator?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.separator?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const separatorVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: separatorDefaults.variant,
    size: separatorDefaults.size,
  },
});
export type SeparatorVariants = VariantProps<typeof separatorVariants>;

export const uiParts: Record<string, string> = {
  root: 'shrink-0 border-outline data-[akaza-orientation=horizontal]:border-t data-[akaza-orientation=vertical]:border-l',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function separatorUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? separatorDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, separatorVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
