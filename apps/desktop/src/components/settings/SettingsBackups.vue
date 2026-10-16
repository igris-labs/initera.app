<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useFileDialog } from '@vueuse/core';
import { useWorkspaceStore } from '../../stores/workspace';
import { notifications } from '../../lib/notifications';
import { usesNativeStorage } from '../../lib/workspace-storage';
import {
  createBackup,
  discardImport,
  exportBackupToFile,
  exportPortableToFile,
  finishImport,
  inspectImportFile,
  listBackups,
  restoreBackup,
} from '../../lib/backup-storage';
import type { BackupEntry, ImportSummary } from '../../lib/backup-storage';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';

const workspace = useWorkspaceStore();
const backups = ref<BackupEntry[]>([]);
const backupBusy = ref(false);
const backupError = ref('');
const exportedPath = ref('');
const restoreTarget = ref<BackupEntry | null>(null);
const restoreOpen = ref(false);
const importOpen = ref(false);
const importFile = ref<File | null>(null);
const importUpload = ref<{
  id: string;
  kind: 'backup' | 'portable';
  summary: ImportSummary;
} | null>(null);
const { open: chooseBackupFile, onChange: onBackupFileChange } = useFileDialog({
  accept: '.sqlite,.zip,application/x-sqlite3,application/zip',
  multiple: false,
});

async function copyExportPath() {
  try {
    await navigator.clipboard.writeText(exportedPath.value);
    notifications.add({ title: 'Export path copied', type: 'success' });
  } catch {
    backupError.value = 'Cannot copy export path.';
  }
}

onBackupFileChange(async (files) => {
  importFile.value = files?.item(0) ?? null;
  if (!importFile.value) return;
  const file = importFile.value;
  const kind = file.name.toLowerCase().endsWith('.zip') ? 'portable' : 'backup';
  const limit = kind === 'portable' ? 550 : 100;
  if (file.size > limit * 1024 * 1024) {
    backupError.value = `Choose an export smaller than ${limit} MB.`;
    return;
  }

  backupBusy.value = true;
  backupError.value = '';
  try {
    if (importUpload.value) await discardImport(importUpload.value.id);
    importUpload.value = await inspectImportFile(file, kind);
    importOpen.value = true;
  } catch (error) {
    backupError.value = String(error);
  } finally {
    backupBusy.value = false;
  }
});

async function cancelImport() {
  importOpen.value = false;
  const upload = importUpload.value;
  importUpload.value = null;
  importFile.value = null;
  if (upload) await discardImport(upload.id).catch(() => {});
}

function onImportDialogOpenChange(open: boolean) {
  if (!open) void cancelImport();
}

onMounted(async () => {
  if (!usesNativeStorage) return;
  try {
    backups.value = await listBackups();
  } catch (error) {
    backupError.value = String(error);
  }
});

async function saveBackup() {
  backupBusy.value = true;
  backupError.value = '';
  try {
    await createBackup();
    backups.value = await listBackups();
    notifications.add({ title: 'Backup created', type: 'success' });
  } catch (error) {
    backupError.value = String(error);
  } finally {
    backupBusy.value = false;
  }
}

async function confirmRestore() {
  if (!restoreTarget.value) return;
  backupBusy.value = true;
  backupError.value = '';
  try {
    await restoreBackup(restoreTarget.value.id);
    location.reload();
  } catch (error) {
    backupError.value = String(error);
    restoreOpen.value = false;
  } finally {
    backupBusy.value = false;
  }
}

async function exportBackup(backup: BackupEntry) {
  backupBusy.value = true;
  backupError.value = '';
  try {
    const path = await exportBackupToFile(backup.id);
    if (!path) return;
    exportedPath.value = path;
    notifications.add({
      title: 'Backup exported',
      description: exportedPath.value,
      type: 'success',
    });
  } catch (error) {
    backupError.value = String(error);
  } finally {
    backupBusy.value = false;
  }
}

async function exportWorkspace() {
  backupBusy.value = true;
  backupError.value = '';

  try {
    const path = await exportPortableToFile();
    if (!path) return;
    exportedPath.value = path;
    notifications.add({
      title: 'Workspace exported',
      description: exportedPath.value,
      type: 'success',
    });
  } catch (error) {
    backupError.value = String(error);
  } finally {
    backupBusy.value = false;
  }
}

async function confirmImport() {
  if (!importUpload.value) return;

  backupBusy.value = true;
  backupError.value = '';
  try {
    await finishImport(
      importUpload.value.id,
      importUpload.value.kind,
      importUpload.value.summary.checksum,
    );
    location.reload();
  } catch (error) {
    backupError.value = String(error);
    importOpen.value = false;
  } finally {
    backupBusy.value = false;
  }
}
</script>

<template>
  <section v-if="usesNativeStorage" aria-labelledby="backup-heading">
    <p v-if="workspace.restoreError" class="mb-3 text-xs text-danger" role="alert">
      Backup restore failed: {{ workspace.restoreError }} Your previous data is still available.
    </p>
    <div class="flex items-center justify-between gap-3">
      <div>
        <h3 id="backup-heading" class="text-sm font-medium text-foreground-highlighted">Backups</h3>
        <p class="mt-1 text-xs text-foreground-muted">
          Save workspace database and managed images together in your data folder.
        </p>
      </div>
      <div class="flex items-center gap-1">
        <Button
          label="Export workspace"
          variant="ghost"
          size="sm"
          :disabled="backupBusy"
          @click="exportWorkspace"
        />
        <Button
          label="Import"
          variant="ghost"
          size="sm"
          :disabled="backupBusy"
          @click="chooseBackupFile()"
        />
        <Button label="Create backup" size="sm" :disabled="backupBusy" @click="saveBackup" />
      </div>
    </div>
    <p v-if="backupError" class="mt-3 text-xs text-danger" role="alert">{{ backupError }}</p>
    <p v-if="exportedPath" class="mt-3 break-all text-xs text-foreground-muted" role="status">
      Saved to {{ exportedPath }}
      <Button label="Copy path" size="xs" variant="ghost" @click="copyExportPath" />
    </p>
    <ul v-if="backups.length" class="mt-3 divide-y divide-outline border-y border-outline">
      <li
        v-for="backup in backups"
        :key="backup.id"
        class="flex min-h-11 items-center justify-between gap-3 py-2"
      >
        <span class="text-xs text-foreground-muted"
          >{{ new Date(backup.createdAt * 1000).toLocaleString() }} ·
          {{ Math.ceil(backup.bytes / 1024) }} KB</span
        >
        <div class="flex items-center gap-1">
          <Button
            label="Export"
            variant="ghost"
            size="xs"
            :disabled="backupBusy"
            @click="exportBackup(backup)"
          />
          <Button
            label="Restore"
            variant="ghost"
            size="xs"
            :disabled="backupBusy"
            @click="
              restoreTarget = backup;
              restoreOpen = true;
            "
          />
        </div>
      </li>
    </ul>
    <p v-else class="mt-3 text-xs text-foreground-muted">No backups yet.</p>
  </section>
  <Dialog
    v-model:open="restoreOpen"
    title="Restore backup"
    description="The app will restart to replace current data. A recovery copy is kept in your data folder."
    size="sm"
  >
    <template #body>
      <p class="text-sm text-foreground">
        Restore backup from
        {{ restoreTarget ? new Date(restoreTarget.createdAt * 1000).toLocaleString() : '' }}?
      </p>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="restoreOpen = false" />
      <Button label="Restore backup" size="sm" :disabled="backupBusy" @click="confirmRestore" />
    </template>
  </Dialog>
  <Dialog
    :open="importOpen"
    title="Import workspace"
    description="After validation, the app will restart to replace current data. A recovery copy is kept."
    size="sm"
    @update:open="onImportDialogOpenChange"
  >
    <template #body>
      <div v-if="importUpload" class="space-y-2 text-sm text-foreground">
        <p class="font-medium">{{ importUpload.summary.workspaceName }}</p>
        <p class="break-all text-xs text-foreground-muted">{{ importFile?.name }}</p>
        <p class="text-xs text-foreground-muted">
          {{ importUpload.summary.projects }} projects · {{ importUpload.summary.issues }} issues ·
          {{ importUpload.summary.cycles }} cycles · {{ importUpload.summary.images }} images
        </p>
        <p>Current workspace will be replaced after confirmation.</p>
      </div>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="cancelImport" />
      <Button label="Import workspace" size="sm" :disabled="backupBusy" @click="confirmImport" />
    </template>
  </Dialog>
</template>
