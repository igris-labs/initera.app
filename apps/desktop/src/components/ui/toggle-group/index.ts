import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const toggleGroupDefaults = {
  size: uiConfig.components?.toggleGroup?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.toggleGroup?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const toggleGroupVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: toggleGroupDefaults.variant,
    size: toggleGroupDefaults.size,
  },
});
export type ToggleGroupVariants = VariantProps<typeof toggleGroupVariants>;
export { default as ToggleGroup } from './ToggleGroup.vue';

export const uiParts: Record<string, string> = {
  root: 'inline-flex items-center gap-0.5 rounded-lg bg-muted p-0.5',
  item: 'min-h-6 rounded-md px-2 text-xs data-[akaza-state=on]:bg-default data-[akaza-state=on]:shadow-sm data-[akaza-state=on]:ring-1 data-[akaza-state=on]:ring-outline',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    item: 'text-xs min-h-7 px-2',
  },
  md: {
    item: 'text-sm min-h-8 px-2.5',
  },
  lg: {
    item: 'text-base min-h-9 px-3',
  },
};

export function toggleGroupUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? toggleGroupDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, toggleGroupVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
