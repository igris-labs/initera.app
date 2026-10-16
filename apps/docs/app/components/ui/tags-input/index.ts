import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const tagsInputDefaults = {
  size: uiConfig.components?.tagsInput?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.tagsInput?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const tagsInputVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: tagsInputDefaults.variant,
    size: tagsInputDefaults.size,
  },
});
export type TagsInputVariants = VariantProps<typeof tagsInputVariants>;
export { default as TagsInput } from './TagsInput.vue';

export const uiParts: Record<string, string> = {
  root: 'flex min-h-7 flex-wrap items-center gap-1 rounded-lg border border-outline bg-default px-2',
  item: 'inline-flex items-center gap-1 rounded-md bg-muted px-1.5 py-0.5 text-xs',
  input: 'min-w-16 flex-1 bg-transparent text-sm outline-none',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'min-h-7 gap-1.5 px-2.5 text-xs',
    input: 'text-xs',
    item: 'text-xs',
  },
  md: {
    root: 'min-h-8 gap-1.5 px-3 text-sm',
    input: 'text-sm',
    item: 'text-sm',
  },
  lg: {
    root: 'min-h-9 gap-2 px-3.5 text-base',
    input: 'text-base',
    item: 'text-base',
  },
};

export function tagsInputUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? tagsInputDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, tagsInputVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
