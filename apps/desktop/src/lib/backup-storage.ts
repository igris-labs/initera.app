import { invoke } from './native-invoke';

export interface BackupEntry {
  id: string;
  createdAt: number;
  bytes: number;
}

export interface ImportSummary {
  checksum: string;
  workspaceName: string;
  projects: number;
  issues: number;
  cycles: number;
  images: number;
}

export function listBackups() {
  return invoke<BackupEntry[]>('storage_list_backups');
}

export function createBackup() {
  return invoke<BackupEntry>('storage_create_backup');
}

export function restoreBackup(id: string) {
  return invoke<void>('storage_restore_backup', { id });
}

export function exportPortableToFile() {
  return invoke<string | null>('storage_export_portable_file');
}

export function exportBackupToFile(id: string) {
  return invoke<string | null>('storage_export_backup_file', { id });
}

export async function inspectImportFile(file: File, kind: 'backup' | 'portable') {
  const id = await invoke<string>('storage_begin_import_upload');
  try {
    const chunkSize = 256 * 1024;
    for (let offset = 0; offset < file.size; offset += chunkSize) {
      const bytes = new Uint8Array(await file.slice(offset, offset + chunkSize).arrayBuffer());
      let binary = '';
      for (let start = 0; start < bytes.length; start += 32_768) {
        binary += String.fromCharCode(...bytes.subarray(start, start + 32_768));
      }

      await invoke('storage_append_import_chunk', { id, offset, chunk: btoa(binary) });
    }

    const summary = await invoke<ImportSummary>('storage_inspect_import_upload', { id, kind });
    return { id, kind, summary };
  } catch (error) {
    await invoke('storage_discard_import_upload', { id }).catch(() => {});
    throw error;
  }
}

export function discardImport(id: string) {
  return invoke<void>('storage_discard_import_upload', { id });
}

export function finishImport(id: string, kind: 'backup' | 'portable', checksum: string) {
  return invoke<void>('storage_finish_import_upload', { id, kind, checksum });
}

export function runRestoreCatchup() {
  return invoke<{ processed: number; total: number; done: boolean }>('storage_run_restore_catchup');
}
