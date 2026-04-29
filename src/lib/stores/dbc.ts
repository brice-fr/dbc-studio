// DBC model store with undo/redo support.
import { writable, get } from 'svelte/store';
import type { DbcModel, MessageModel, SignalModel, NodeModel } from '../types';
import { emptyModel, newMessage, newSignal } from '../types';

// ─── Undo / redo stack ────────────────────────────────────────────────────────

const MAX_HISTORY = 100;

function createDbcStore() {
  const history: DbcModel[] = [emptyModel()];
  let cursor = 0;

  const { subscribe, set } = writable<DbcModel>(history[0]);

  function push(next: DbcModel) {
    // Drop any redo states above cursor
    history.splice(cursor + 1);
    history.push(structuredClone(next));
    if (history.length > MAX_HISTORY) history.shift();
    cursor = history.length - 1;
    set(next);
  }

  return {
    subscribe,

    /** Replace the entire model (e.g. after loading a file). */
    load(model: DbcModel) {
      history.length = 0;
      history.push(structuredClone(model));
      cursor = 0;
      set(model);
    },

    undo() {
      if (cursor > 0) {
        cursor--;
        set(structuredClone(history[cursor]));
      }
    },

    redo() {
      if (cursor < history.length - 1) {
        cursor++;
        set(structuredClone(history[cursor]));
      }
    },

    canUndo: () => cursor > 0,
    canRedo: () => cursor < history.length - 1,

    // ─── Message CRUD ─────────────────────────────────────────────────────

    addMessage(msg: MessageModel) {
      const next = structuredClone(get({ subscribe }));
      next.messages.push(msg);
      push(next);
    },

    updateMessage(id: number, patch: Partial<MessageModel>) {
      const next = structuredClone(get({ subscribe }));
      const idx = next.messages.findIndex((m) => m.id === id);
      if (idx !== -1) {
        next.messages[idx] = { ...next.messages[idx], ...patch };
        push(next);
      }
    },

    deleteMessage(id: number) {
      const next = structuredClone(get({ subscribe }));
      next.messages = next.messages.filter((m) => m.id !== id);
      push(next);
    },

    // ─── Signal CRUD ──────────────────────────────────────────────────────

    addSignal(messageId: number, sig: SignalModel) {
      const next = structuredClone(get({ subscribe }));
      const msg = next.messages.find((m) => m.id === messageId);
      if (msg) {
        msg.signals.push(sig);
        push(next);
      }
    },

    updateSignal(messageId: number, signalName: string, patch: Partial<SignalModel>) {
      const next = structuredClone(get({ subscribe }));
      const msg = next.messages.find((m) => m.id === messageId);
      if (msg) {
        const sigIdx = msg.signals.findIndex((s) => s.name === signalName);
        if (sigIdx !== -1) {
          msg.signals[sigIdx] = { ...msg.signals[sigIdx], ...patch };
          push(next);
        }
      }
    },

    deleteSignal(messageId: number, signalName: string) {
      const next = structuredClone(get({ subscribe }));
      const msg = next.messages.find((m) => m.id === messageId);
      if (msg) {
        msg.signals = msg.signals.filter((s) => s.name !== signalName);
        push(next);
      }
    },

    // ─── Node CRUD ────────────────────────────────────────────────────────

    addNode(node: NodeModel) {
      const next = structuredClone(get({ subscribe }));
      next.nodes.push(node);
      push(next);
    },

    updateNode(name: string, patch: Partial<NodeModel>) {
      const next = structuredClone(get({ subscribe }));
      const idx = next.nodes.findIndex((n) => n.name === name);
      if (idx !== -1) {
        next.nodes[idx] = { ...next.nodes[idx], ...patch };
        push(next);
      }
    },

    deleteNode(name: string) {
      const next = structuredClone(get({ subscribe }));
      next.nodes = next.nodes.filter((n) => n.name !== name);
      push(next);
    },

    // ─── Version ──────────────────────────────────────────────────────────

    setVersion(version: string) {
      const next = structuredClone(get({ subscribe }));
      next.version = version;
      push(next);
    },
  };
}

export const dbcStore = createDbcStore();

// ─── Dirty flag ───────────────────────────────────────────────────────────────
// Tracks whether the model has unsaved changes.

export const isDirty = writable(false);
export const currentFilePath = writable<string | null>(null);

// Mark dirty whenever the model changes after initial load.
let initialized = false;
dbcStore.subscribe(() => {
  if (initialized) isDirty.set(true);
  else initialized = true;
});

export function markClean(path: string) {
  currentFilePath.set(path);
  isDirty.set(false);
}
