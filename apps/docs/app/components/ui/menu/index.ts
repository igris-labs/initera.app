import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Menu } from './Menu.vue';
export const menuDefaults = {
  size: uiConfig.components?.menu?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.menu?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const menuVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: menuDefaults.variant,
    size: menuDefaults.size,
  },
});
export type MenuVariants = VariantProps<typeof menuVariants>;

export const uiParts: Record<string, string> = {
  content: 'z-50 rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
  item: 'flex cursor-pointer items-center rounded-md text-foreground outline-none transition-colors duration-100 hover:bg-accented focus-visible:bg-accented data-akaza-disabled:cursor-not-allowed data-akaza-disabled:opacity-40',
  separator: 'my-1.5 border-t border-outline',
  label: 'px-2.5 py-1.5 text-xs font-semibold text-foreground-muted',
  submenuContent: 'z-50 rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    item: 'min-h-7 px-2 text-xs',
    content: 'p-1 min-w-36',
    submenuContent: 'p-1 min-w-36',
    label: 'px-2 py-1',
  },
  md: {
    item: 'min-h-8 px-2.5 text-sm',
    content: 'p-1.5 min-w-44',
    submenuContent: 'p-1.5 min-w-44',
    label: 'px-2.5 py-1.5',
  },
  lg: {
    item: 'min-h-9 px-3 text-base',
    content: 'p-2 min-w-56',
    submenuContent: 'p-2 min-w-56',
    label: 'px-3 py-2',
  },
};

export function menuUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? menuDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(styles.content, menuVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
