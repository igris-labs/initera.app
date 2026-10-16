import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as HoverPreviewCard } from './HoverPreviewCard.vue';
export const hoverPreviewCardDefaults = {
  size: uiConfig.components?.hoverPreviewCard?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant:
    uiConfig.components?.hoverPreviewCard?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const hoverPreviewCardVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: hoverPreviewCardDefaults.variant,
    size: hoverPreviewCardDefaults.size,
  },
});
export type HoverPreviewCardVariants = VariantProps<typeof hoverPreviewCardVariants>;

export const uiParts: Record<string, string> = {
  content: 'z-50 max-w-sm rounded-lg border border-outline bg-elevated p-3 text-sm shadow-lg',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function hoverPreviewCardUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? hoverPreviewCardDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    hoverPreviewCardVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
