import { invoke } from '@tauri-apps/api/core';
import type { Engine, PrivacySettings, SearchHit, Session, SessionDetail } from './types';

export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export class DeskError extends Error {
  constructor(message = 'Run make dev to talk to the local store.') {
    super(message);
    this.name = 'DeskError';
  }
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new DeskError();
  }
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    const obj = typeof e === 'object' && e !== null ? (e as Record<string, unknown>) : null;
    const msg = obj && 'message' in obj ? String(obj.message) : String(e);
    const hint = obj && 'action_hint' in obj ? String(obj.action_hint) : '';
    throw new DeskError(hint ? `${msg} ${hint}` : msg || 'The local store rejected that command.');
  }
}

export const api = {
  settingsGet: (key: string) => call<string | null>('settings_get', { key }),
  settingsSet: (key: string, value: string) => call<void>('settings_set', { key, value }),
  engines: () => call<Engine[]>('engines_list'),
  sessions: () => call<Session[]>('sessions_list', { limit: 50 }),
  session: (sessionId: string) => call<SessionDetail>('sessions_get', { sessionId }),
  start: (title?: string, source = 'mic') =>
    call<Session>('recorder_start', { args: { title, source } }),
  consent: (sessionId: string) => call<Session>('recorder_consent', { sessionId }),
  begin: (sessionId: string) => call<Session>('recorder_begin', { sessionId }),
  pause: (sessionId: string) => call<Session>('recorder_pause', { sessionId }),
  resume: (sessionId: string) => call<Session>('recorder_resume', { sessionId }),
  stop: (sessionId: string) => call<Session>('recorder_stop', { sessionId }),
  transcribe: (sessionId: string, modelId?: string) =>
    call<SessionDetail>('transcribe_run', { sessionId, modelId }),
  search: (
    q: string,
    filters?: { title?: string; createdFrom?: string; createdTo?: string; tag?: string },
  ) =>
    call<SearchHit[]>('search_query', {
      q,
      limit: 20,
      title: filters?.title,
      createdFrom: filters?.createdFrom,
      createdTo: filters?.createdTo,
      tag: filters?.tag,
    }),
  setTags: (sessionId: string, tags: string[]) =>
    call<string[]>('sessions_set_tags', { sessionId, tags }),
  rename: (sessionId: string, title: string) =>
    call<Session>('sessions_rename', { sessionId, title }),
  exportMd: (sessionId: string) => call<string>('sessions_export', { sessionId }),
  exportFile: (sessionId: string, dest: string) =>
    call<void>('sessions_export_file', { sessionId, dest }),
  privacy: () => call<PrivacySettings>('privacy_settings'),
  deleteSession: (sessionId: string) => call<void>('sessions_delete', { sessionId }),
  installModelFile: (engineId: string, path: string, expectedSha256: string) =>
    call<{ engine_id: string; bytes_written: number; sha256: string }>('model_install_file', {
      engineId,
      path,
      expectedSha256,
    }),
  deleteModel: (engineId: string) => call<void>('model_delete', { engineId }),
  deleteAll: () => call<void>('data_delete_all'),
  keyReport: () => call<{ backend: string; key_len: number; fingerprint: string }>('key_report'),
  applyRetention: () => call<number>('retention_apply'),
  loginGet: () =>
    call<{ backend: string; requested: boolean; applied: boolean }>('presence_login_get'),
  loginSet: (enabled: boolean) =>
    call<{ backend: string; requested: boolean; applied: boolean }>('presence_login_set', {
      enabled,
    }),
  hotkeyGet: () => call<{ shortcut: string; mode: string }>('hotkey_get'),
  hotkeySet: (shortcut: string, mode: string) =>
    call<{ shortcut: string; mode: string }>('hotkey_set', { shortcut, mode }),
};

export function formatClock(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const m = String(Math.floor(total / 60)).padStart(2, '0');
  const s = String(total % 60).padStart(2, '0');
  return `${m}:${s}`;
}

export function formatStamp(ms: number): string {
  const m = Math.floor(ms / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  return `${m}:${String(s).padStart(2, '0')}`;
}
