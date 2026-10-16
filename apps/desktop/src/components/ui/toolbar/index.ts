import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const toolbarDefaults = {
  size: uiConfig.components?.toolbar?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.toolbar?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const toolbarVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: toolbarDefaults.variant,
    size: toolbarDefaults.size,
  },
});
export type ToolbarVariants = VariantProps<typeof toolbarVariants>;
export { default as Toolbar } from './Toolbar.vue';

export const uiParts: Record<string, string> = {
  root: 'flex items-center gap-1 rounded-lg border border-outline bg-default p-1',
  button: 'min-h-7 rounded-md px-2 text-xs hover:bg-muted',
  separator: 'mx-1 h-5 border-l border-outline',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    button: 'text-xs min-h-7 px-2',
  },
  md: {
    button: 'text-sm min-h-8 px-2.5',
  },
  lg: {
    button: 'text-base min-h-9 px-3',
  },
};

export function toolbarUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? toolbarDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, toolbarVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
