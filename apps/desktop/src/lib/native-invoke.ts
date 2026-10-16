import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import type { InvokeArgs } from '@tauri-apps/api/core';
import { trackWrite } from './pending-writes';

export type ApplicationErrorCode =
  | 'VALIDATION'
  | 'NOT_FOUND'
  | 'ARCHIVED'
  | 'CONFLICT'
  | 'HIERARCHY_CYCLE'
  | 'CROSS_PROJECT'
  | 'CYCLE_SCOPE'
  | 'IDENTIFIER_CONFLICT'
  | 'TIMER_RUNNING'
  | 'CONFIRMATION_REQUIRED'
  | 'INVALID_CONTENT'
  | 'MEDIA_FAILURE'
  | 'STORAGE_FULL'
  | 'DATABASE_BUSY'
  | 'MIGRATION_FAILED'
  | 'UNSUPPORTED_VERSION'
  | 'IMPORT_INVALID'
  | 'IDEMPOTENCY_CONFLICT'
  | 'INTERNAL';

export class ApplicationError extends Error {
  constructor(
    readonly code: ApplicationErrorCode,
    message: string,
    readonly retryable: boolean,
    readonly fieldErrors?: Record<string, string>,
    readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'ApplicationError';
  }
}

interface ApiResponse<T> {
  data: T;
  workspaceRevision: number;
  changed?: { id: string; kind: string }[];
  removedFromActive?: { id: string; kind: string }[];
}

const sideEffectCommands = new Set([
  'storage_create_backup',
  'storage_restore_backup',
  'storage_export_portable_file',
  'storage_export_backup_file',
  'storage_begin_import_upload',
  'storage_append_import_chunk',
  'storage_discard_import_upload',
  'storage_finish_import_upload',
  'storage_run_restore_catchup',
  'storage_stage_image',
]);

const errorCodes: ReadonlySet<string> = new Set<ApplicationErrorCode>([
  'VALIDATION',
  'NOT_FOUND',
  'ARCHIVED',
  'CONFLICT',
  'HIERARCHY_CYCLE',
  'CROSS_PROJECT',
  'CYCLE_SCOPE',
  'IDENTIFIER_CONFLICT',
  'TIMER_RUNNING',
  'CONFIRMATION_REQUIRED',
  'INVALID_CONTENT',
  'MEDIA_FAILURE',
  'STORAGE_FULL',
  'DATABASE_BUSY',
  'MIGRATION_FAILED',
  'UNSUPPORTED_VERSION',
  'IMPORT_INVALID',
  'IDEMPOTENCY_CONFLICT',
  'INTERNAL',
]);

function applicationError(error: unknown): ApplicationError {
  if (error instanceof ApplicationError) return error;
  if (error && typeof error === 'object' && 'code' in error && 'message' in error) {
    const value = error as {
      code: string;
      message: string;
      retryable?: boolean;
      fieldErrors?: Record<string, string>;
      details?: Record<string, unknown>;
    };
    if (errorCodes.has(value.code))
      return new ApplicationError(
        value.code as ApplicationErrorCode,
        value.message,
        value.retryable ?? false,
        value.fieldErrors,
        value.details,
      );
  }

  const message = String(error);
  const separator = message.indexOf(': ');
  const prefix = separator > 0 ? message.slice(0, separator) : '';
  if (errorCodes.has(prefix)) {
    const code = prefix as ApplicationErrorCode;
    return new ApplicationError(
      code,
      message.slice(separator + 2),
      ['CONFLICT', 'DATABASE_BUSY'].includes(code),
    );
  }

  const safeMessage =
    message.length <= 240 && !/[\\/\n\r]/.test(message)
      ? message
      : 'The operation could not be completed.';
  const validation =
    /^(?:Choose|Enter|Keep|Select|Invalid|Unknown|Too many|Issue query contains)/.test(message);

  return new ApplicationError(validation ? 'VALIDATION' : 'INTERNAL', safeMessage, !validation);
}

let workspaceId = '';

export function setMutationWorkspace(id: string) {
  workspaceId = id;
}

async function requestKey(command: string, args: Record<string, unknown>) {
  // Omit the generated ID so the same unacknowledged request finds its old ID after reload.
  const mutation = args.mutation as Record<string, unknown>;
  const request = JSON.stringify({
    workspaceId,
    command,
    ...args,
    mutation: { ...mutation, mutationId: undefined },
  });
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(request));
  return `pm-app:pending-mutation:${Array.from(new Uint8Array(digest), (byte) =>
    byte.toString(16).padStart(2, '0'),
  ).join('')}`;
}

async function resolveVersionKeys(versions: Record<string, number>) {
  // Some UI callers know issue identifiers, while native version checks use entity UUIDs.
  const uuid = /^[\da-f]{8}-(?:[\da-f]{4}-){3}[\da-f]{12}$/i;
  const identifiers = Object.keys(versions).filter((key) => !uuid.test(key));
  if (!identifiers.length) return versions;

  const resolved: Record<string, string> = {};
  for (let start = 0; start < identifiers.length; start += 1000) {
    const response = await tauriInvoke<ApiResponse<Record<string, string>>>(
      'storage_resolve_issue_entity_ids_api',
      {
        request: {
          contractVersion: 1,
          requestId: crypto.randomUUID(),
          input: { identifiers: identifiers.slice(start, start + 1000) },
        },
      },
    );
    Object.assign(resolved, response.data);
  }

  return Object.fromEntries(
    Object.entries(versions).map(([key, version]) => [resolved[key] ?? key, version]),
  );
}

// Keep an unacknowledged ID across retries, including reload after an IPC response is lost.
async function invokeInner<T>(command: string, args?: InvokeArgs): Promise<T> {
  const input = (args ?? {}) as Record<string, unknown>;
  const mutation = input.mutation as Record<string, unknown> | undefined;
  const mutationId =
    (mutation?.mutationId as string | undefined) ??
    (sideEffectCommands.has(command) ? crypto.randomUUID() : undefined);
  const versions = mutation?.versions as Record<string, number> | undefined;
  const expectedVersion = mutation?.expectedVersion as number | null | undefined;
  const target = [
    ...(command === 'storage_start_timer' || command === 'storage_stop_timer'
      ? [workspaceId]
      : command === 'storage_restore_milestone'
        ? [input.projectId]
        : command.includes('planning_node')
          ? [input.projectId]
          : []),
    input.id,
    input.entryId,
    input.commentId,
    input.cycleId,
    input.seriesId,
    input.milestoneId,
    input.projectId,
    input.identifier,
    (input.view as { id?: string } | undefined)?.id,
    (input.settings as { id?: string } | undefined)?.id,
    workspaceId,
  ].find((value): value is string => typeof value === 'string' && value.length > 0);
  const expectedVersionsInput: Record<string, number> = {
    ...(versions ??
      (typeof expectedVersion === 'number' && target ? { [target]: expectedVersion } : {})),
  };
  const targets = input.targets as { identifier: string; version: number }[] | undefined;
  for (const item of targets ?? []) expectedVersionsInput[item.identifier] = item.version;
  if (typeof input.targetVersion === 'number' && typeof input.targetIdentifier === 'string') {
    expectedVersionsInput[input.targetIdentifier] = input.targetVersion;
  }

  if (typeof input.nextCycleVersion === 'number' && typeof input.nextCycleId === 'string') {
    expectedVersionsInput[input.nextCycleId] = input.nextCycleVersion;
  }

  if (!mutationId) {
    try {
      const response = await tauriInvoke<ApiResponse<T>>(`${command}_api`, {
        request: { contractVersion: 1, requestId: crypto.randomUUID(), input },
      });
      return response.data;
    } catch (error) {
      throw applicationError(error);
    }
  }

  const key = await requestKey(command, mutation ? input : { ...input, mutation: { mutationId } });
  let id = mutationId;

  try {
    id = sessionStorage.getItem(key) ?? id;
    sessionStorage.setItem(key, id);
  } catch {
    // Session storage can be disabled; current invocation still keeps its ID.
  }

  let result: ApiResponse<T>;
  try {
    const expectedVersions = await resolveVersionKeys(expectedVersionsInput);
    result = await tauriInvoke<ApiResponse<T>>(`${command}_api`, {
      request: {
        contractVersion: 1,
        mutationId: id,
        expectedVersions,
        input: mutation ? { ...input, mutation: { ...mutation, mutationId: id } } : input,
      },
    });
  } catch (error) {
    throw applicationError(error);
  }

  try {
    sessionStorage.removeItem(key);
  } catch {
    // A successful native commit remains authoritative.
  }

  return result.data;
}

export function invoke<T>(command: string, args?: InvokeArgs): Promise<T> {
  const mutation = (args as Record<string, unknown> | undefined)?.mutation;
  const result = invokeInner<T>(command, args);
  return mutation || sideEffectCommands.has(command) ? trackWrite(result) : result;
}
