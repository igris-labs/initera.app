import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const tooltipDefaults = {
  size: uiConfig.components?.tooltip?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.tooltip?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const tooltipVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: tooltipDefaults.variant,
    size: tooltipDefaults.size,
  },
});
export type TooltipVariants = VariantProps<typeof tooltipVariants>;
export { default as Tooltip } from './Tooltip.vue';

export const uiParts: Record<string, string> = {
  content:
    'z-50 rounded-md border border-outline bg-elevated px-2.5 py-1 text-xs text-foreground shadow-sm',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function tooltipUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? tooltipDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    tooltipVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
