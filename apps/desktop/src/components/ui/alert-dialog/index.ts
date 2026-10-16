import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as AlertDialog } from './AlertDialog.vue';
export const alertDialogDefaults = {
  size: uiConfig.components?.alertDialog?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.alertDialog?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const alertDialogVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: alertDialogDefaults.variant,
    size: alertDialogDefaults.size,
  },
});
export type AlertDialogVariants = VariantProps<typeof alertDialogVariants>;

export const uiParts: Record<string, string> = {
  overlay: 'fixed inset-0 bg-scrim',
  content:
    'fixed top-1/2 left-1/2 z-50 w-[min(32rem,calc(100vw-2rem))] -translate-x-1/2 -translate-y-1/2 rounded-xl border border-outline bg-elevated p-4 text-foreground shadow-lg',
  header: 'grid gap-1.5',
  title: 'font-semibold',
  description: 'text-sm text-foreground-muted',
  body: 'py-3 text-sm text-foreground',
  footer: 'flex justify-end gap-2 pt-4',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    title: 'text-sm',
    description: 'text-xs',
  },
  md: {
    title: 'text-base',
    description: 'text-xs',
  },
  lg: {
    title: 'text-lg',
    description: 'text-sm',
  },
};

export function alertDialogUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? alertDialogDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    alertDialogVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
