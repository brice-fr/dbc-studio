// UI state store — selection, panels, notifications.
import { writable } from 'svelte/store';
import type { ValidationIssue } from '../types';

// ─── Selection ────────────────────────────────────────────────────────────────

export const selectedMessageId = writable<number | null>(null);
export const selectedSignalName = writable<string | null>(null);

export function selectMessage(id: number | null) {
  selectedMessageId.set(id);
  selectedSignalName.set(null);
}

export function selectSignal(messageId: number, signalName: string) {
  selectedMessageId.set(messageId);
  selectedSignalName.set(signalName);
}

// ─── Validation panel ─────────────────────────────────────────────────────────

export const validationIssues = writable<ValidationIssue[]>([]);
export const showValidationPanel = writable(false);

// ─── Toast notifications ──────────────────────────────────────────────────────

export interface Toast {
  id: number;
  type: 'success' | 'error' | 'info';
  message: string;
}

export const toasts = writable<Toast[]>([]);
let toastCounter = 0;

export function showToast(type: Toast['type'], message: string, durationMs = 3000) {
  const id = ++toastCounter;
  toasts.update((t) => [...t, { id, type, message }]);
  setTimeout(() => toasts.update((t) => t.filter((x) => x.id !== id)), durationMs);
}

// ─── Layout ───────────────────────────────────────────────────────────────────

export const treePanelWidth = writable(260);
export const propertiesPanelWidth = writable(280);

// ─── Display mode ─────────────────────────────────────────────────────────────

/** When true, CAN IDs are shown in hex (0x1A2); when false, in decimal (418). */
export const hexMode = writable<boolean>(true);
