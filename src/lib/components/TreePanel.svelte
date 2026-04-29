<script lang="ts">
  import { dbcStore } from '../stores/dbc';
  import { selectedMessageId, selectedSignalName, selectMessage, selectSignal, showToast, hexMode } from '../stores/ui';
  import { newSignal, formatCanId } from '../types';
  import type { MessageModel } from '../types';

  // Which messages are expanded
  let expanded: Set<number> = new Set();

  function toggle(id: number) {
    if (expanded.has(id)) expanded.delete(id);
    else expanded.add(id);
    expanded = expanded; // trigger reactivity
  }

  function handleSelectMessage(id: number) {
    selectMessage(id);
    if (!expanded.has(id)) toggle(id);
  }

  function handleAddSignal(msg: MessageModel) {
    // Generate a unique signal name
    const used = new Set(msg.signals.map((s) => s.name));
    let base = 'Signal';
    let i = 1;
    while (used.has(`${base}_${i}`)) i++;
    const sig = newSignal();
    sig.name = `${base}_${i}`;
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

  // Drag-and-drop state (future)
  let dragOverMsgId: number | null = null;
</script>

<div class="tree-panel">
  <div class="panel-header">
    <span>Messages ({$dbcStore.messages.length})</span>
  </div>

  <div class="tree-scroll">
    {#if $dbcStore.messages.length === 0}
      <div class="empty">No messages. Click <strong>+ Message</strong> to add one.</div>
    {/if}

    {#each $dbcStore.messages as msg (msg.id)}
      {@const isExpanded = expanded.has(msg.id)}
      {@const isMsgSelected = $selectedMessageId === msg.id && $selectedSignalName === null}

      <!-- Message row -->
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
        <!-- Expand arrow -->
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

        <!-- Context actions -->
        <div class="row-actions">
          <button
            class="action-btn"
            title="Add signal"
            on:click|stopPropagation={() => handleAddSignal(msg)}
          >+</button>
          <button
            class="action-btn delete"
            title="Delete message"
            on:click|stopPropagation={() => handleDeleteMessage(msg.id, msg.name)}
          >✕</button>
        </div>
      </div>

      <!-- Signals (when expanded) -->
      {#if isExpanded}
        {#each msg.signals as sig (sig.name)}
          {@const isSigSelected = $selectedMessageId === msg.id && $selectedSignalName === sig.name}
          <div
            class="sig-row"
            class:selected={isSigSelected}
            on:click={() => selectSignal(msg.id, sig.name)}
            on:keydown={(e) => e.key === 'Enter' && selectSignal(msg.id, sig.name)}
            role="treeitem"
            tabindex="0"
            aria-selected={isSigSelected}
          >
            <span class="sig-indent"></span>
            <span class="sig-icon">{sig.is_multiplexer ? 'M' : '~'}</span>
            <span class="sig-name">{sig.name}</span>
            <span class="sig-meta">{sig.start_bit}|{sig.length}</span>
            <div class="row-actions">
              <button
                class="action-btn delete"
                title="Delete signal"
                on:click|stopPropagation={() => handleDeleteSignal(msg.id, sig.name)}
              >✕</button>
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

    <!-- Nodes section -->
    {#if $dbcStore.nodes.length > 0}
      <div class="section-header">Nodes ({$dbcStore.nodes.length})</div>
      {#each $dbcStore.nodes as node}
        <div class="node-row">
          <span class="node-icon">⬡</span>
          <span>{node.name}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .tree-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
  }
  .panel-header {
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
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
  /* Message rows */
  .msg-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px 3px 4px;
    font-size: 12px;
    cursor: pointer;
    user-select: none;
    position: relative;
  }
  .msg-row:hover { background: var(--bg-hover); }
  .msg-row.selected { background: var(--bg-selected); color: var(--text-selected); }
  .expand-btn {
    background: none; border: none; cursor: pointer;
    padding: 0 2px; font-size: 10px; color: var(--text-muted);
    width: 14px; flex-shrink: 0;
  }
  .msg-icon { flex-shrink: 0; font-size: 11px; }
  .msg-name { font-weight: 500; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .msg-id { font-size: 10px; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
  .msg-dlc { font-size: 10px; color: var(--text-muted); flex-shrink: 0; }
  /* Signal rows */
  .sig-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px 2px 0;
    font-size: 11px;
    cursor: pointer;
    user-select: none;
  }
  .sig-row:hover { background: var(--bg-hover); }
  .sig-row.selected { background: var(--bg-selected); color: var(--text-selected); }
  .sig-indent { width: 28px; flex-shrink: 0; }
  .sig-icon { font-size: 10px; flex-shrink: 0; color: var(--text-muted); width: 14px; text-align: center; }
  .sig-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sig-meta { font-size: 10px; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
  .empty-signals { pointer-events: none; padding-left: 8px; }
  /* Action buttons */
  .row-actions {
    display: none;
    gap: 2px;
    margin-left: auto;
    flex-shrink: 0;
  }
  .msg-row:hover .row-actions,
  .sig-row:hover .row-actions,
  .msg-row.selected .row-actions,
  .sig-row.selected .row-actions { display: flex; }
  .action-btn {
    background: none; border: none; cursor: pointer;
    padding: 1px 4px; font-size: 11px;
    border-radius: 3px; color: var(--text-muted);
  }
  .action-btn:hover { background: var(--bg-badge); }
  .action-btn.delete:hover { background: #e53e3e20; color: #e53e3e; }
  /* Node rows */
  .section-header {
    padding: 8px 10px 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
    margin-top: 4px;
  }
  .node-row {
    display: flex; align-items: center; gap: 6px;
    padding: 3px 10px;
    font-size: 12px;
    color: var(--text-muted);
  }
  .node-icon { font-size: 10px; }
  .muted { color: var(--text-muted); font-style: italic; }
</style>
