// Tauri invoke() wrappers — typed bindings to Rust commands.
import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen, save as dialogSave } from '@tauri-apps/plugin-dialog';
import type { DbcModel, MessageModel, SignalModel, NodeModel, ValidationIssue } from './types';

// ─── File dialogs ─────────────────────────────────────────────────────────────

export async function pickOpenFile(): Promise<string | null> {
  const result = await dialogOpen({
    title: 'Open DBC File',
    filters: [{ name: 'DBC Files', extensions: ['dbc'] }],
    multiple: false,
  });
  if (Array.isArray(result)) return result[0] ?? null;
  return result ?? null;
}

export async function pickSaveFile(defaultName?: string): Promise<string | null> {
  const result = await dialogSave({
    title: 'Save DBC File',
    defaultPath: defaultName,
    filters: [{ name: 'DBC Files', extensions: ['dbc'] }],
  });
  return result ?? null;
}

// ─── DBC I/O ─────────────────────────────────────────────────────────────────

export async function openDbc(path: string): Promise<DbcModel> {
  return invoke<DbcModel>('open_dbc', { path });
}

export async function saveDbc(path: string, model: DbcModel): Promise<void> {
  return invoke<void>('save_dbc', { path, model });
}

// ─── Model helpers ────────────────────────────────────────────────────────────

export async function defaultMessage(
  id: number,
  name: string,
  dlc: number,
  sender: string
): Promise<MessageModel> {
  return invoke<MessageModel>('default_message', { args: { id, name, dlc, sender } });
}

export async function defaultSignal(name: string): Promise<SignalModel> {
  return invoke<SignalModel>('default_signal', { args: { name } });
}

export async function defaultNode(name: string): Promise<NodeModel> {
  return invoke<NodeModel>('default_node', { name });
}

// ─── Validation ───────────────────────────────────────────────────────────────

export async function validateDbc(model: DbcModel): Promise<ValidationIssue[]> {
  return invoke<ValidationIssue[]>('validate_dbc', { model });
}
