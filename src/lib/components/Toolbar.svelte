<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dbcStore, isDirty, currentFilePath, markClean } from '../stores/dbc';
  import { showToast, hexMode } from '../stores/ui';
  import { pickOpenFile, pickSaveFile, openDbc, saveDbc,
           getRecentFiles, addRecentFile, clearRecentFiles } from '../api';
  import { emptyModel, newMessage } from '../types';

  const dispatch = createEventDispatcher();

  let saving  = false;
  let opening = false;

  // ── Recent files dropdown ──────────────────────────────────────────────────
  let showRecent  = false;
  let recentFiles: string[] = [];

  async function toggleRecent() {
    if (!showRecent) recentFiles = await getRecentFiles();
    showRecent = !showRecent;
  }

  async function handleClearRecent() {
    await clearRecentFiles();
    recentFiles = [];
    showRecent = false;
  }

  function basename(p: string) {
    return p.split(/[\\/]/).pop() ?? p;
  }

  // ── Core open-path helper (shared by Open, Open Recent, file-association) ──
  async function openPath(path: string) {
    const { model, warnings } = await openDbc(path);
    dbcStore.load(model);
    markClean(path);
    addRecentFile(path); // fire-and-forget
    showToast('success', `Opened ${basename(path)}`);
    for (const w of warnings) showToast('info', w, 6000);
  }

  async function handleOpen() {
    if ($isDirty) {
      const ok = confirm('You have unsaved changes. Open a new file anyway?');
      if (!ok) return;
    }
    opening = true;
    try {
      const path = await pickOpenFile();
      if (!path) return;
      await openPath(path);
    } catch (e) {
      showToast('error', String(e));
    } finally {
      opening = false;
    }
  }

  async function handleOpenRecent(path: string) {
    showRecent = false;
    if ($isDirty) {
      const ok = confirm('You have unsaved changes. Open a new file anyway?');
      if (!ok) return;
    }
    opening = true;
    try {
      await openPath(path);
    } catch (e) {
      showToast('error', String(e));
      // File may have been deleted — refresh the list
      recentFiles = await getRecentFiles();
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
        addRecentFile(path);
      }
      await saveDbc(path, $dbcStore);
      markClean(path);
      showToast('success', 'Saved');
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
      addRecentFile(path);
      showToast('success', `Saved as ${basename(path)}`);
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
    const used = new Set($dbcStore.messages.map((m) => m.id));
    let id = 0x100;
    while (used.has(id)) id++;
    dbcStore.addMessage(newMessage(id));
  }

  function handleUndo() { dbcStore.undo(); }
  function handleRedo() { dbcStore.redo(); }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { showRecent = false; return; }
    if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) { e.preventDefault(); handleUndo(); }
    if ((e.metaKey || e.ctrlKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) { e.preventDefault(); handleRedo(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 's' && !e.shiftKey) { e.preventDefault(); handleSave(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 's' && e.shiftKey) { e.preventDefault(); handleSaveAs(); }
    if ((e.metaKey || e.ctrlKey) && e.key === 'o') { e.preventDefault(); handleOpen(); }
  }

  function onWindowClick(e: MouseEvent) {
    if (showRecent && !(e.target as Element)?.closest('.recent-wrap')) {
      showRecent = false;
    }
  }
</script>

<svelte:window on:keydown={onKeydown} on:click={onWindowClick} />

<div class="toolbar">
  <!-- File group -->
  <div class="group">
    <button class="btn" on:click={handleNew} title="New (Ctrl+N)">
      <span>📄</span> New
    </button>
    <button class="btn" on:click={handleOpen} disabled={opening} title="Open (Ctrl+O)">
      <span>📂</span> Open
    </button>
    <!-- Recent files dropdown -->
    <div class="recent-wrap">
      <button class="btn recent-trigger" on:click={toggleRecent}
        title="Open a recently opened file" aria-haspopup="true" aria-expanded={showRecent}>
        Recent ▾
      </button>
      {#if showRecent}
        <div class="recent-dropdown" role="menu">
          {#if recentFiles.length === 0}
            <div class="recent-empty">No recent files</div>
          {:else}
            {#each recentFiles as path (path)}
              <button class="recent-item" role="menuitem"
                title={path} on:click={() => handleOpenRecent(path)}>
                <span class="recent-name">{basename(path)}</span>
                <span class="recent-dir">{path.split(/[\\/]/).slice(0, -1).join('/')}</span>
              </button>
            {/each}
            <div class="recent-separator"></div>
            <button class="recent-clear" role="menuitem" on:click={handleClearRecent}>
              Clear list
            </button>
          {/if}
        </div>
      {/if}
    </div>
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

  /* ── Recent files dropdown ─────────────────────────────────────────────── */
  .recent-wrap { position: relative; }
  .recent-trigger { font-size: 12px; }
  .recent-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    z-index: 200;
    min-width: 280px;
    max-width: 420px;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0,0,0,.12);
    padding: 4px 0;
    display: flex;
    flex-direction: column;
  }
  .recent-empty {
    padding: 8px 14px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .recent-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    padding: 5px 14px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  .recent-item:hover { background: var(--bg-hover); }
  .recent-name {
    font-size: 12px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .recent-dir {
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
    direction: rtl; /* truncates from the left, keeping the deepest folder visible */
  }
  .recent-separator {
    height: 1px;
    background: var(--border-light);
    margin: 4px 0;
  }
  .recent-clear {
    padding: 5px 14px;
    font-size: 11px;
    color: var(--text-muted);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  .recent-clear:hover { color: var(--text); background: var(--bg-hover); }
</style>
