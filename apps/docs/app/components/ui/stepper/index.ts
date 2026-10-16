import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Stepper } from './Stepper.vue';
export const stepperDefaults = {
  size: uiConfig.components?.stepper?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.stepper?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const stepperVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: stepperDefaults.variant,
    size: stepperDefaults.size,
  },
});
export type StepperVariants = VariantProps<typeof stepperVariants>;

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  list: 'flex items-center gap-2',
  trigger: 'inline-flex items-center gap-2 rounded-md px-2 py-1 hover:bg-muted',
  indicator: 'flex size-6 items-center justify-center rounded-full border border-outline',
  title: 'font-medium',
  description: 'text-xs text-foreground-muted',
  separator: 'h-px flex-1 bg-outline',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    title: 'text-xs',
    description: 'text-xs',
    indicator: 'size-6',
  },
  md: {
    title: 'text-sm',
    description: 'text-xs',
    indicator: 'size-7',
  },
  lg: {
    title: 'text-base',
    description: 'text-sm',
    indicator: 'size-8',
  },
};

export function stepperUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? stepperDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, stepperVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
