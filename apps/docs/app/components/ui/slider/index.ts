import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Slider } from './Slider.vue';
export const sliderDefaults = {
  size: uiConfig.components?.slider?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.slider?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const sliderVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: sliderDefaults.variant,
    size: sliderDefaults.size,
  },
});
export type SliderVariants = VariantProps<typeof sliderVariants>;

export const uiParts: Record<string, string> = {
  root: 'relative flex min-h-7 items-center',
  track: 'h-2 w-full rounded-full bg-muted',
  range: 'h-full rounded-full bg-inverted',
  thumb:
    'size-4 rounded-full border border-outline bg-default shadow-sm focus-visible:outline-2 focus-visible:outline-inverted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    track: 'h-1.5',
    thumb: 'size-3.5',
  },
  md: {
    track: 'h-2',
    thumb: 'size-4',
  },
  lg: {
    track: 'h-3',
    thumb: 'size-5',
  },
};

export function sliderUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? sliderDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, sliderVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
