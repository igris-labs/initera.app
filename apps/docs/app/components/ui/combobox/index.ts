import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Combobox } from './Combobox.vue';
export const comboboxDefaults = {
  size: uiConfig.components?.combobox?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.combobox?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const comboboxVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: comboboxDefaults.variant,
    size: comboboxDefaults.size,
  },
});
export type ComboboxVariants = VariantProps<typeof comboboxVariants>;

export const uiParts: Record<string, string> = {
  root: 'relative text-sm',
  input:
    'w-full rounded-lg border border-outline bg-default px-2.5 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-inverted/15',
  content: 'z-50 rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
  option:
    'flex cursor-pointer items-center rounded-md text-foreground outline-none transition-colors duration-100 hover:bg-accented focus-visible:bg-accented data-akaza-disabled:cursor-not-allowed data-akaza-disabled:opacity-40',
  empty: 'p-2 text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    option: 'min-h-7 px-2 text-xs',
    content: 'p-1 min-w-36',
    input: 'min-h-7 gap-1.5 px-2.5 text-xs',
  },
  md: {
    option: 'min-h-8 px-2.5 text-sm',
    content: 'p-1.5 min-w-44',
    input: 'min-h-8 gap-1.5 px-3 text-sm',
  },
  lg: {
    option: 'min-h-9 px-3 text-base',
    content: 'p-2 min-w-56',
    input: 'min-h-9 gap-2 px-3.5 text-base',
  },
};

export function comboboxUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? comboboxDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    comboboxVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
