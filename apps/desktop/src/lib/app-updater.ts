import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import type { Update } from '@tauri-apps/plugin-updater';
import { ref, shallowRef } from 'vue';
import { invoke } from './native-invoke';
import { notifications } from './notifications';
import { flushPendingWrites } from './pending-writes';
import { usesNativeStorage } from './workspace-storage';

export const updateAvailable = shallowRef<Update | null>(null);
export const updateOpen = ref(false);
export const updateChecking = ref(false);
export const updateInstalling = ref(false);
export const updateError = ref('');
export const updateProgress = ref('');

export async function checkForUpdates(manual = false) {
  if (!usesNativeStorage || updateChecking.value || updateInstalling.value) return;

  updateChecking.value = true;
  updateError.value = '';

  try {
    const update = await check();
    if (updateAvailable.value) await updateAvailable.value.close();
    updateAvailable.value = update;
    updateOpen.value = Boolean(update);

    if (!update && manual) {
      notifications.add({ title: 'Initera is up to date', type: 'success' });
    }
  } catch (error) {
    if (manual) {
      notifications.add({
        title: 'Could not check for updates',
        description: String(error),
        type: 'error',
      });
    }
  } finally {
    updateChecking.value = false;
  }
}

export async function installUpdate() {
  const update = updateAvailable.value;
  if (!update || updateInstalling.value) return;

  updateInstalling.value = true;
  updateError.value = '';
  updateProgress.value = 'Downloading update…';

  try {
    await update.download();
    updateProgress.value = 'Saving your work…';
    await flushPendingWrites();
    await invoke('storage_prepare_update');
    updateProgress.value = 'Installing update…';
    await update.install();
    await relaunch();
  } catch (error) {
    updateError.value = String(error);
    updateProgress.value = '';
  } finally {
    updateInstalling.value = false;
  }
}

export function dismissUpdate() {
  if (updateInstalling.value) return;
  updateOpen.value = false;
  void updateAvailable.value?.close();
  updateAvailable.value = null;
}
