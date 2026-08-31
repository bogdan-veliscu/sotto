import { invoke } from '@tauri-apps/api/core';
import type { Engine, SearchHit, Session, SessionDetail } from './types';

export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

export const api = {
  settingsGet: (key: string) => call<string | null>('settings_get', { key }),
  settingsSet: (key: string, value: string) => call<void>('settings_set', { key, value }),
  engines: () => call<Engine[]>('engines_list'),
  sessions: () => call<Session[]>('sessions_list', { limit: 50 }),
  session: (sessionId: string) => call<SessionDetail>('sessions_get', { sessionId }),
  start: (title?: string, source = 'mixed') =>
    call<Session>('recorder_start', { args: { title, source } }),
  consent: (sessionId: string) => call<Session>('recorder_consent', { sessionId }),
  begin: (sessionId: string) => call<Session>('recorder_begin', { sessionId }),
  pause: (sessionId: string) => call<Session>('recorder_pause', { sessionId }),
  resume: (sessionId: string) => call<Session>('recorder_resume', { sessionId }),
  stopFixture: (sessionId: string) => call<Session>('recorder_stop_fixture', { sessionId }),
  transcribe: (sessionId: string, modelId?: string) =>
    call<SessionDetail>('transcribe_run', { sessionId, modelId }),
  search: (q: string) => call<SearchHit[]>('search_query', { q, limit: 20 }),
  rename: (sessionId: string, title: string) =>
    call<Session>('sessions_rename', { sessionId, title }),
  exportMd: (sessionId: string) => call<string>('sessions_export', { sessionId }),
  deleteSession: (sessionId: string) => call<void>('sessions_delete', { sessionId }),
  deleteAll: () => call<void>('data_delete_all'),
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
