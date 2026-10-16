import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Editable } from './Editable.vue';
export const editableDefaults = {
  size: uiConfig.components?.editable?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.editable?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const editableVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: editableDefaults.variant,
    size: editableDefaults.size,
  },
});
export type EditableVariants = VariantProps<typeof editableVariants>;

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  preview: 'rounded-md px-2 py-1 hover:bg-muted',
  input:
    'rounded-lg border border-outline bg-default px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-inverted/15',
  actions: 'flex items-center gap-1',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    input: 'min-h-7 gap-1.5 px-2.5 text-xs',
  },
  md: {
    input: 'min-h-8 gap-1.5 px-3 text-sm',
  },
  lg: {
    input: 'min-h-9 gap-2 px-3.5 text-base',
  },
};

export function editableUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? editableDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, editableVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
