import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Listbox } from './Listbox.vue';
export const listboxDefaults = {
  size: uiConfig.components?.listbox?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.listbox?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const listboxVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: listboxDefaults.variant,
    size: listboxDefaults.size,
  },
});
export type ListboxVariants = VariantProps<typeof listboxVariants>;

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  content: 'z-50 rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
  option:
    'flex cursor-pointer items-center rounded-md text-foreground outline-none transition-colors duration-100 hover:bg-accented focus-visible:bg-accented data-akaza-disabled:cursor-not-allowed data-akaza-disabled:opacity-40',
  empty: 'p-2 text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    option: 'min-h-7 px-2 text-xs',
    content: 'p-1 min-w-36',
  },
  md: {
    option: 'min-h-8 px-2.5 text-sm',
    content: 'p-1.5 min-w-44',
  },
  lg: {
    option: 'min-h-9 px-3 text-base',
    content: 'p-2 min-w-56',
  },
};

export function listboxUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? listboxDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    listboxVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
