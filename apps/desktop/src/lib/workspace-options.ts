import { invoke } from './native-invoke';
import { usesNativeStorage } from './workspace-storage';

export type ProjectMovePreference = 'ask' | 'keep' | 'regenerate';

export async function getProjectMovePreference(): Promise<ProjectMovePreference> {
  if (!usesNativeStorage) return 'ask';
  const options = await invoke<{ projectMoveIdentifier: ProjectMovePreference }>(
    'storage_get_workspace_options',
  );
  return options.projectMoveIdentifier;
}

export async function setProjectMovePreference(preference: ProjectMovePreference): Promise<void> {
  if (!usesNativeStorage) return;
  await invoke('storage_set_project_move_preference', {
    preference,
    mutation: { mutationId: crypto.randomUUID() },
  });
}
