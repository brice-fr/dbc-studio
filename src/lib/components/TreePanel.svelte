<script lang="ts">
  import { dbcStore } from '../stores/dbc';
  import {
    selectedMessageId, selectedSignalName,
    selectMessage, selectSignal, selectNode, selectedNodeName,
    showToast, hexMode,
  } from '../stores/ui';
  import { newSignal, formatCanId } from '../types';
  import type { MessageModel } from '../types';

  // ─── Expansion state ──────────────────────────────────────────────────────
  let expanded: Set<number> = new Set();

  function toggle(id: number) {
    if (expanded.has(id)) expanded.delete(id);
    else expanded.add(id);
    expanded = expanded;
  }

  // ─── Search / filter ─────────────────────────────────────────────────────
  let searchQuery = '';

  $: filteredMessages = (() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return $dbcStore.messages;
    return $dbcStore.messages.filter(
      (m) =>
        m.name.toLowerCase().includes(q) ||
        m.signals.some((s) => s.name.toLowerCase().includes(q)),
    );
  })();

  // Auto-expand messages that have signal matches when searching
  $: if (searchQuery.trim()) {
    const q = searchQuery.trim().toLowerCase();
    for (const msg of $dbcStore.messages) {
      if (msg.signals.some((s) => s.name.toLowerCase().includes(q))) {
        expanded.add(msg.id);
      }
    }
    expanded = expanded;
  }

  function clearSearch() { searchQuery = ''; }

  // ─── Message helpers ──────────────────────────────────────────────────────
  function handleSelectMessage(id: number) {
    selectMessage(id);
    if (!expanded.has(id)) toggle(id);
  }

  function handleAddSignal(msg: MessageModel) {
    const used = new Set(msg.signals.map((s) => s.name));
    let i = 1;
    while (used.has(`Signal_${i}`)) i++;
    const sig = newSignal();
    sig.name = `Signal_${i}`;
    dbcStore.addSignal(msg.id, sig);
    selectSignal(msg.id, sig.name);
  }

  function handleDeleteMessage(id: number, name: string) {
    if (!confirm(`Delete message "${name}"?`)) return;
    dbcStore.deleteMessage(id);
    if ($selectedMessageId === id) selectMessage(null);
    showToast('info', `Deleted message "${name}"`);
  }

  function handleDeleteSignal(msgId: number, sigName: string) {
    if (!confirm(`Delete signal "${sigName}"?`)) return;
    dbcStore.deleteSignal(msgId, sigName);
    if ($selectedMessageId === msgId && $selectedSignalName === sigName) selectMessage(msgId);
    showToast('info', `Deleted signal "${sigName}"`);
  }

  // ─── Node helpers ─────────────────────────────────────────────────────────
  let addingNode = false;
  let newNodeName = '';
  let renamingNode: string | null = null;
  let renameValue = '';

  function handleSelectNode(name: string) {
    selectNode(name);
  }

  function startAddNode() {
    addingNode = true;
    newNodeName = '';
  }

  function commitAddNode() {
    const name = newNodeName.trim();
    if (!name) { addingNode = false; return; }
    if ($dbcStore.nodes.some((n) => n.name === name)) {
      showToast('error', `Node "${name}" already exists`);
      return;
    }
    dbcStore.addNode({ name, comment: null });
    showToast('success', `Added node "${name}"`);
    addingNode = false;
    newNodeName = '';
    selectNode(name);
  }

  function cancelAddNode() { addingNode = false; newNodeName = ''; }

  function startRename(name: string) {
    renamingNode = name;
    renameValue = name;
  }

  function commitRename(oldName: string) {
    const name = renameValue.trim();
    if (!name || name === oldName) { renamingNode = null; return; }
    if ($dbcStore.nodes.some((n) => n.name === name)) {
      showToast('error', `Node "${name}" already exists`);
      return;
    }
    dbcStore.updateNode(oldName, { name });
    showToast('success', `Renamed node to "${name}"`);
    if ($selectedNodeName === oldName) selectNode(name);
    renamingNode = null;
  }

  function handleDeleteNode(name: string) {
    // Find everything that references this node
    const txMessages = $dbcStore.messages.filter((m) => m.sender === name);
    const rxSignals: { msgName: string; sigName: string }[] = [];
    for (const msg of $dbcStore.messages) {
      for (const sig of msg.signals) {
        if (sig.receivers.includes(name)) {
          rxSignals.push({ msgName: msg.name, sigName: sig.name });
        }
      }
    }

    let msg = `Delete node "${name}"?`;
    if (txMessages.length > 0 || rxSignals.length > 0) {
      msg += '\n\nThis node is currently referenced:';
      if (txMessages.length > 0) {
        msg += `\n\nTransmitter of ${txMessages.length} message${txMessages.length > 1 ? 's' : ''}:`;
        msg += '\n' + txMessages.map((m) => `  • ${m.name}`).join('\n');
      }
      if (rxSignals.length > 0) {
        msg += `\n\nReceiver of ${rxSignals.length} signal${rxSignals.length > 1 ? 's' : ''}:`;
        msg += '\n' + rxSignals.map((s) => `  • ${s.msgName} → ${s.sigName}`).join('\n');
      }
      msg += '\n\nAll affected transmitter / receiver fields will be reset to None.';
    }

    if (!confirm(msg)) return;
    dbcStore.deleteNode(name);
    if ($selectedNodeName === name) selectNode(null);
    showToast('info', `Deleted node "${name}"`);
  }
</script>

<div class="tree-panel">
  <!-- ── Fixed header with search (applies to messages/signals) ───────────── -->
  <div class="panel-header">
    <span class="header-label">Tree</span>
    <div class="search-wrap">
      <input
        class="search-input"
        type="text"
        placeholder="Filter messages…"
        bind:value={searchQuery}
        aria-label="Filter messages and signals"
      />
      {#if searchQuery}
        <button class="search-clear" on:click={clearSearch} title="Clear filter">✕</button>
      {/if}
    </div>
  </div>

  <div class="tree-scroll">

    <!-- ── Nodes section (top) ───────────────────────────────────────────── -->
    <div class="section-header section-header-top">
      <span>Nodes ({$dbcStore.nodes.length})</span>
      <button class="section-add-btn" title="Add node" on:click={startAddNode}>+</button>
    </div>

    {#each $dbcStore.nodes as node (node.name)}
      {@const isNodeSelected = $selectedNodeName === node.name}
      <div
        class="node-row"
        class:selected={isNodeSelected}
        on:click={() => handleSelectNode(node.name)}
        on:keydown={(e) => e.key === 'Enter' && handleSelectNode(node.name)}
        role="treeitem"
        tabindex="0"
        aria-selected={isNodeSelected}
      >
        <span class="node-icon">⬡</span>
        {#if renamingNode === node.name}
          <input
            class="rename-input"
            bind:value={renameValue}
            on:keydown={(e) => {
              if (e.key === 'Enter') commitRename(node.name);
              if (e.key === 'Escape') renamingNode = null;
            }}
            on:blur={() => commitRename(node.name)}
            use:focusOnMount
          />
        {:else}
          <span class="node-name">{node.name}</span>
        {/if}
        <div class="row-actions">
          <button class="action-btn" title="Rename node" on:click|stopPropagation={() => startRename(node.name)}>✎</button>
          <button class="action-btn delete" title="Delete node" on:click|stopPropagation={() => handleDeleteNode(node.name)}>✕</button>
        </div>
      </div>
    {/each}

    {#if addingNode}
      <div class="node-row add-node-row">
        <span class="node-icon">⬡</span>
        <input
          class="rename-input"
          placeholder="Node name"
          bind:value={newNodeName}
          on:keydown={(e) => {
            if (e.key === 'Enter') commitAddNode();
            if (e.key === 'Escape') cancelAddNode();
          }}
          on:blur={commitAddNode}
          use:focusOnMount
        />
      </div>
    {/if}

    {#if $dbcStore.nodes.length === 0 && !addingNode}
      <div class="node-row empty-nodes">
        <span class="muted">No nodes. Click + to add.</span>
      </div>
    {/if}

    <!-- ── Messages section (below nodes) ───────────────────────────────── -->
    <div class="section-header">
      <span>Messages ({$dbcStore.messages.length})</span>
    </div>

    {#if filteredMessages.length === 0}
      <div class="empty">
        {#if searchQuery}No results for "{searchQuery}".{:else}No messages. Click <strong>+ Message</strong> to add one.{/if}
      </div>
    {/if}

    {#each filteredMessages as msg (msg.id)}
      {@const isExpanded = expanded.has(msg.id)}
      {@const isMsgSelected = $selectedMessageId === msg.id && $selectedSignalName === null}
      {@const q = searchQuery.trim().toLowerCase()}

      <div
        class="msg-row"
        class:selected={isMsgSelected}
        on:click={() => handleSelectMessage(msg.id)}
        on:keydown={(e) => e.key === 'Enter' && handleSelectMessage(msg.id)}
        role="treeitem"
        tabindex="0"
        aria-expanded={isExpanded}
        aria-selected={isMsgSelected}
      >
        <button
          class="expand-btn"
          on:click|stopPropagation={() => toggle(msg.id)}
          aria-label={isExpanded ? 'Collapse' : 'Expand'}
        >
          {isExpanded ? '▾' : '▸'}
        </button>
        <span class="msg-icon">✉</span>
        <span class="msg-name">{msg.name}</span>
        <span class="msg-id">{formatCanId(msg, $hexMode)}</span>
        <span class="msg-dlc">[{msg.dlc}]</span>
        <div class="row-actions">
          <button class="action-btn" title="Add signal" on:click|stopPropagation={() => handleAddSignal(msg)}>+</button>
          <button class="action-btn delete" title="Delete message" on:click|stopPropagation={() => handleDeleteMessage(msg.id, msg.name)}>✕</button>
        </div>
      </div>

      {#if isExpanded}
        {#each msg.signals as sig (sig.name)}
          {@const isSigSelected = $selectedMessageId === msg.id && $selectedSignalName === sig.name}
          {@const isMatch = q && sig.name.toLowerCase().includes(q)}
          <div
            class="sig-row"
            class:selected={isSigSelected}
            class:search-match={isMatch}
            on:click={() => selectSignal(msg.id, sig.name)}
            on:keydown={(e) => e.key === 'Enter' && selectSignal(msg.id, sig.name)}
            role="treeitem"
            tabindex="0"
            aria-selected={isSigSelected}
          >
            <span class="sig-indent"></span>
            <span class="sig-icon">
              {#if sig.is_multiplexer}M{:else if sig.multiplexer_switch_value !== null}m{:else}~{/if}
            </span>
            <span class="sig-name">{sig.name}</span>
            {#if sig.multiplexer_switch_value !== null}
              <span class="sig-mux-tag">m{sig.multiplexer_switch_value}</span>
            {/if}
            <span class="sig-meta">{sig.start_bit}|{sig.length}</span>
            <div class="row-actions">
              <button class="action-btn delete" title="Delete signal" on:click|stopPropagation={() => handleDeleteSignal(msg.id, sig.name)}>✕</button>
            </div>
          </div>
        {/each}

        {#if msg.signals.length === 0}
          <div class="sig-row empty-signals">
            <span class="sig-indent"></span>
            <span class="muted">No signals</span>
          </div>
        {/if}
      {/if}
    {/each}

  </div>
</div>

<!-- Svelte action: focus the element on mount -->
<script context="module" lang="ts">
  function focusOnMount(node: HTMLElement) {
    node.focus();
    (node as HTMLInputElement).select?.();
    return {};
  }
</script>

<style>
  .tree-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
  }
  /* ── Header ─────────────────────────────────────────────────────────────── */
  .panel-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px 5px 10px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    white-space: nowrap;
  }
  /* ── Search ─────────────────────────────────────────────────────────────── */
  .search-wrap {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    position: relative;
  }
  .search-input {
    flex: 1;
    min-width: 0;
    height: 22px;
    padding: 0 20px 0 6px;
    font-size: 11px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input);
    color: var(--text);
    outline: none;
  }
  .search-input:focus { border-color: var(--accent); }
  .search-clear {
    position: absolute;
    right: 4px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 9px;
    color: var(--text-muted);
    padding: 0 2px;
    line-height: 1;
  }
  .search-clear:hover { color: var(--text); }
  /* ── Scroll area ─────────────────────────────────────────────────────────── */
  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 0;
  }
  .empty {
    padding: 16px 12px;
    font-size: 12px;
    color: var(--text-muted);
    text-align: center;
  }
  /* ── Message rows ────────────────────────────────────────────────────────── */
  .msg-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px 3px 4px;
    font-size: 12px;
    cursor: pointer;
    user-select: none;
    position: relative;
    outline: none;
  }
  .msg-row:hover { background: var(--bg-hover); }
  .msg-row.selected { background: var(--bg-selected); color: var(--text-selected); }
  .expand-btn {
    background: none; border: none; cursor: pointer;
    padding: 0 2px; font-size: 10px; color: var(--text-muted);
    width: 14px; flex-shrink: 0;
  }
  .msg-row.selected .expand-btn { color: var(--text-selected); }
  .msg-icon { flex-shrink: 0; font-size: 11px; }
  .msg-name { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .msg-id   { font-size: 10px; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
  .msg-dlc  { font-size: 10px; color: var(--text-muted); flex-shrink: 0; }
  .msg-row.selected .msg-id,
  .msg-row.selected .msg-dlc { color: rgba(255,255,255,0.7); }
  /* ── Signal rows ─────────────────────────────────────────────────────────── */
  .sig-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px 2px 0;
    font-size: 11px;
    cursor: pointer;
    user-select: none;
    outline: none;
  }
  .sig-row:hover { background: var(--bg-hover); }
  .sig-row.selected { background: var(--bg-selected); color: var(--text-selected); }
  .sig-row.search-match { background: #fef9c3; }
  .sig-row.search-match.selected { background: var(--bg-selected); }
  .sig-indent { width: 28px; flex-shrink: 0; }
  .sig-icon { font-size: 10px; flex-shrink: 0; color: var(--text-muted); width: 14px; text-align: center; }
  .sig-row.selected .sig-icon { color: rgba(255,255,255,0.7); }
  .sig-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sig-mux-tag {
    font-size: 9px;
    font-family: monospace;
    background: #dbeafe;
    color: #1d4ed8;
    border-radius: 3px;
    padding: 0 3px;
    flex-shrink: 0;
  }
  .sig-row.selected .sig-mux-tag { background: rgba(255,255,255,0.2); color: #fff; }
  .sig-meta { font-size: 10px; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
  .sig-row.selected .sig-meta { color: rgba(255,255,255,0.7); }
  .empty-signals, .empty-nodes { pointer-events: none; padding-left: 8px; }
  /* ── Action buttons ──────────────────────────────────────────────────────── */
  .row-actions {
    display: none;
    gap: 2px;
    margin-left: auto;
    flex-shrink: 0;
  }
  .msg-row:hover .row-actions,
  .sig-row:hover .row-actions,
  .node-row:hover .row-actions,
  .msg-row.selected .row-actions,
  .sig-row.selected .row-actions,
  .node-row.selected .row-actions { display: flex; }
  .action-btn {
    background: none; border: none; cursor: pointer;
    padding: 1px 4px; font-size: 11px;
    border-radius: 3px; color: var(--text-muted);
  }
  .action-btn:hover { background: var(--bg-badge); }
  .action-btn.delete:hover { background: #e53e3e20; color: #e53e3e; }
  /* ── Node section ────────────────────────────────────────────────────────── */
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 4px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
    margin-top: 4px;
  }
  /* Top section needs no top border/margin since it's flush with the panel header */
  .section-header-top {
    border-top: none;
    margin-top: 0;
    padding-top: 6px;
  }
  .section-add-btn {
    background: none; border: none; cursor: pointer;
    font-size: 14px; color: var(--text-muted);
    padding: 0 4px; line-height: 1; border-radius: 3px;
  }
  .section-add-btn:hover { background: var(--bg-hover); color: var(--text); }
  .node-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 6px 3px 10px;
    font-size: 12px;
    cursor: pointer;
    user-select: none;
    outline: none;
  }
  .node-row:hover { background: var(--bg-hover); }
  .node-row.selected { background: var(--bg-selected); color: var(--text-selected); }
  .node-icon { font-size: 10px; flex-shrink: 0; }
  .node-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  /* ── Inline rename/add inputs ────────────────────────────────────────────── */
  .rename-input {
    flex: 1;
    min-width: 0;
    height: 20px;
    padding: 0 4px;
    font-size: 12px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: var(--bg-input);
    color: var(--text);
    outline: none;
  }
  .muted { color: var(--text-muted); font-style: italic; }
</style>
