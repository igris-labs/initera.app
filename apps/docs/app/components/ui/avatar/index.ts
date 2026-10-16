import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Avatar } from './Avatar.vue';
export const avatarDefaults = {
  size: uiConfig.components?.avatar?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.avatar?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const avatarVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: avatarDefaults.variant,
    size: avatarDefaults.size,
  },
});
export type AvatarVariants = VariantProps<typeof avatarVariants>;

export const uiParts: Record<string, string> = {
  root: 'inline-flex size-8 shrink-0 overflow-hidden rounded-full bg-muted',
  image: 'size-full object-cover',
  fallback: 'flex size-full items-center justify-center text-xs font-medium text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    root: 'size-6',
    fallback: 'text-xs',
  },
  md: {
    root: 'size-8',
    fallback: 'text-sm',
  },
  lg: {
    root: 'size-10',
    fallback: 'text-base',
  },
};

export function avatarUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? avatarDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, avatarVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
