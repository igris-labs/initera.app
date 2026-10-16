import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Accordion } from './Accordion.vue';
export const accordionDefaults = {
  size: uiConfig.components?.accordion?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.accordion?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const accordionVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: accordionDefaults.variant,
    size: accordionDefaults.size,
  },
});
export type AccordionVariants = VariantProps<typeof accordionVariants>;

export const uiParts: Record<string, string> = {
  root: 'divide-y divide-outline',
  item: 'min-w-0',
  trigger:
    'flex w-full items-center justify-between gap-2 rounded-md py-2 text-left text-sm font-medium hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-inverted',
  content: 'pb-3 text-sm text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    trigger: 'text-xs',
    content: 'text-xs',
  },
  md: {
    trigger: 'text-sm',
    content: 'text-sm',
  },
  lg: {
    trigger: 'text-base',
    content: 'text-base',
  },
};

export function accordionUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? accordionDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, accordionVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
