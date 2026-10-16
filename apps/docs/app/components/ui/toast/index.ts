import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const toastDefaults = {
  size: uiConfig.components?.toast?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.toast?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const toastVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: toastDefaults.variant,
    size: toastDefaults.size,
  },
});
export type ToastVariants = VariantProps<typeof toastVariants>;
export { default as Toast } from './Toast.vue';

export const uiParts: Record<string, string> = {
  viewport: 'fixed right-4 bottom-4 z-50 grid gap-2',
  toast:
    'w-[min(22rem,calc(100vw-2rem))] overflow-hidden rounded-lg border border-l-2 border-outline bg-elevated text-xs text-foreground shadow-lg data-[akaza-type=success]:border-l-success data-[akaza-type=error]:border-l-error data-[akaza-type=alert]:border-l-error data-[akaza-type=warning]:border-l-warning data-[akaza-type=info]:border-l-info',
  title: 'font-medium',
  description: 'text-xs text-foreground-muted',
  close: 'rounded-md p-1 hover:bg-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    title: 'text-xs',
    description: 'text-xs',
  },
  md: {
    title: 'text-sm',
    description: 'text-xs',
  },
  lg: {
    title: 'text-base',
    description: 'text-sm',
  },
};

export function toastUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? toastDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.toast = cn(styles.toast, toastVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
