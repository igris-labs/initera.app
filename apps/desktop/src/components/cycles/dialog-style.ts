import type { DialogUi } from 'akaza-ui';

// Match the issue composer so cycle flows share its surface, spacing, and footer treatment.
export const cycleDialogUi: DialogUi = {
  content: 'max-w-2xl',
  header: 'min-h-11 border-b-0 px-4 pt-3 pb-0',
  body: 'px-4 pt-3 pb-3',
  footer: 'border-t-0 px-4 pt-0 pb-3',
};
