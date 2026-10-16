import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as PinInput } from './PinInput.vue';
export const pinInputDefaults = {
  size: uiConfig.components?.pinInput?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.pinInput?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const pinInputVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: pinInputDefaults.variant,
    size: pinInputDefaults.size,
  },
});
export type PinInputVariants = VariantProps<typeof pinInputVariants>;

export const uiParts: Record<string, string> = {
  root: 'flex items-center gap-2',
  input:
    'size-9 rounded-lg border border-outline bg-default text-center text-sm outline-none focus-visible:ring-2 focus-visible:ring-inverted/15',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    input: 'size-7 text-xs',
  },
  md: {
    input: 'size-8 text-sm',
  },
  lg: {
    input: 'size-9 text-base',
  },
};

export function pinInputUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? pinInputDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, pinInputVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
