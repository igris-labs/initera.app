import { createToastManager } from 'akaza-ui';

export const notifications = createToastManager({ duration: 4000 });

export function notifySuccess(title: string, description?: string) {
  notifications.add({ title, description, type: 'success' });
}

let savedTimer: ReturnType<typeof setTimeout> | undefined;

export function notifySaved(context: string) {
  clearTimeout(savedTimer);
  savedTimer = setTimeout(notifySuccess, 700, context);
}
