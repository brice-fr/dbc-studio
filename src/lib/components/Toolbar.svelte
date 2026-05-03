<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dbcStore, isDirty, currentFilePath, markClean } from '../stores/dbc';
  import { showToast, hexMode, showValidationPanel } from '../stores/ui';
  import { pickOpenFile, pickSaveFile, openDbc, saveDbc } from '../api';
  import { emptyModel, newMessage } from '../types';

  const dispatch = createEventDispatcher();

  let saving = false;
  let opening = false;

  async function handleOpen() {
    if ($isDirty) {
      const ok = confirm('You have unsaved changes. Open a new file anyway?');
      if (!ok) return;
    }
    opening = true;
    try {
      const path = await pickOpenFile();
      if (!path) return;
      const { model, warnings } = await openDbc(path);
      dbcStore.load(model);
      markClean(path);
      showToast('success', `Opened ${path.split('/').pop()}`);
      for (const w of warnings) showToast('info', w, 6000);
    } catch (e) {
      showToast('error', String(e));
    } finally {
      opening = false;
    }
  }

  async function handleSave() {
    saving = true;
    try {
      let path = $currentFilePath;
      if (!path) {
        path = await pickSaveFile('untitled.dbc');
        if (!path) return;
      }
      await saveDbc(path, $dbcStore);
      markClean(path);
      showToast('success', 'Saved');
      // Auto-validate after every save
      dispatch('validate');
    } catch (e) {
      showToast('error', String(e));
    } finally {
      saving = false;
    }
  }

  async function handleSaveAs() {
    saving = true;
    try {
      const path = await pickSaveFile($currentFilePath ?? 'untitled.dbc');
      if (!path) return;
      await saveDbc(path, $dbcStore);
      markClean(path);
      showToast('success', `Saved as ${path.split('/').pop()}`);
    } catch (e) {
      showToast('error', String(e));
    } finally {
      saving = false;
    }
  }

  function handleNew() {
    if ($isDirty) {
      const ok = confirm('You have unsaved changes. Create a new file anyway?');
      if (!ok) return;
    }
    dbcStore.load(emptyModel());
    currentFilePath.set(null);
    isDirty.set(false);
  }

  function handleAddMessage() {
    // Find a free CAN ID
    const used = new Set($dbcStore.messages.map((m) => m.id));
    let id = 0x100;
    while (used.has(id)) id++;
    dbcStore.addMessage(newMessage(id));
  }

  function handleUndo() { dbcStore.undo(); }
  function handleRedo() { dbcStore.redo(); }

  // Keyboard shortcuts
  function onKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) { e.preventDefault(); handleUndo(); }
    if ((e.metaKey || e.ctrlKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) { e.preventDefault(); handleRedo(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 's' && !e.shiftKey) { e.preventDefault(); handleSave(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 's' && e.shiftKey) { e.preventDefault(); handleSaveAs(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 'o') { e.preventDefault(); handleOpen(); }
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="toolbar">
  <!-- File group -->
  <div class="group">
    <button class="btn" on:click={handleNew} title="New (Ctrl+N)">
      <span>📄</span> New
    </button>
    <button class="btn" on:click={handleOpen} disabled={opening} title="Open (Ctrl+O)">
      <span>📂</span> Open
    </button>
    <button class="btn" on:click={handleSave} disabled={saving} title="Save (Ctrl+S)">
      <span>💾</span> Save{$isDirty ? ' *' : ''}
    </button>
    <button class="btn" on:click={handleSaveAs} disabled={saving} title="Save As (Ctrl+Shift+S)">
      Save As
    </button>
  </div>

  <div class="separator"></div>

  <!-- Edit group -->
  <div class="group">
    <button class="btn" on:click={handleUndo} title="Undo (Ctrl+Z)">↩ Undo</button>
    <button class="btn" on:click={handleRedo} title="Redo (Ctrl+Y)">↪ Redo</button>
  </div>

  <div class="separator"></div>

  <!-- Model group -->
  <div class="group">
    <button class="btn btn-primary" on:click={handleAddMessage} title="Add Message">
      + Message
    </button>
  </div>

  <div class="separator"></div>

  <!-- Validate -->
  <div class="group">
    <button class="btn" on:click={() => dispatch('validate')} title="Run validation checks">
      ✓ Validate
    </button>
  </div>

  <div class="separator"></div>

  <!-- Display mode toggle -->
  <div class="group">
    <button
      class="btn btn-toggle"
      class:active={$hexMode}
      on:click={() => hexMode.set(true)}
      title="Show IDs in hexadecimal"
    >HEX</button>
    <button
      class="btn btn-toggle"
      class:active={!$hexMode}
      on:click={() => hexMode.set(false)}
      title="Show IDs in decimal"
    >DEC</button>
  </div>

  <!-- File path breadcrumb -->
  <div class="path">
    {#if $currentFilePath}
      <span>{$currentFilePath.split('/').pop()}{$isDirty ? ' ●' : ''}</span>
    {:else}
      <span class="muted">Untitled{$isDirty ? ' ●' : ''}</span>
    {/if}
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--bg-toolbar);
    border-bottom: 1px solid var(--border);
    height: 40px;
    flex-shrink: 0;
  }
  .group { display: flex; gap: 2px; }
  .separator { width: 1px; height: 20px; background: var(--border); margin: 0 4px; }
  .btn {
    display: flex; align-items: center; gap: 4px;
    padding: 4px 10px;
    font-size: 12px;
    border: 1px solid transparent;
    border-radius: 4px;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    white-space: nowrap;
  }
  .btn:hover { background: var(--bg-hover); border-color: var(--border); }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn-primary { background: var(--accent); color: #fff; }
  .btn-primary:hover { background: var(--accent-hover); border-color: transparent; }
  .btn-toggle {
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    border: 1px solid var(--border);
    border-radius: 0;
    color: var(--text-muted);
  }
  .btn-toggle:first-of-type { border-radius: 4px 0 0 4px; }
  .btn-toggle:last-of-type  { border-radius: 0 4px 4px 0; border-left: none; }
  .btn-toggle.active { background: var(--accent); color: #fff; border-color: var(--accent); }
  .btn-toggle:hover:not(.active) { background: var(--bg-hover); }
  .btn-primary:hover { background: var(--accent-hover); border-color: transparent; }
  .path {
    margin-left: auto;
    font-size: 12px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 300px;
  }
  .muted { color: var(--text-muted); }
</style>
