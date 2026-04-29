<script lang="ts">
  import { dbcStore } from '../stores/dbc';
  import { selectedMessageId, selectedSignalName, selectSignal } from '../stores/ui';
  import { newSignal } from '../types';
  import type { SignalModel } from '../types';

  // Derived: current message
  $: selectedMsg = $dbcStore.messages.find((m) => m.id === $selectedMessageId) ?? null;

  // Inline editing state
  type EditCell = { signalName: string; field: keyof SignalModel } | null;
  let editCell: EditCell = null;
  let editValue = '';

  function startEdit(sig: SignalModel, field: keyof SignalModel) {
    editCell = { signalName: sig.name, field };
    editValue = String(sig[field] ?? '');
    // Focus input on next tick
    setTimeout(() => {
      const el = document.querySelector<HTMLInputElement>('.edit-input');
      el?.focus();
      el?.select();
    }, 0);
  }

  function commitEdit() {
    if (!editCell || !selectedMsg) { editCell = null; return; }
    const { signalName, field } = editCell;
    editCell = null;

    let value: unknown = editValue;
    // Coerce types
    const numericFields = new Set<keyof SignalModel>(['start_bit', 'length', 'factor', 'offset', 'min', 'max']);
    const booleanFields = new Set<keyof SignalModel>(['is_unsigned', 'is_multiplexer']);
    if (numericFields.has(field)) value = parseFloat(editValue) || 0;
    if (field === 'start_bit' || field === 'length') value = parseInt(editValue) || 0;
    if (booleanFields.has(field)) value = editValue === 'true';
    if (field === 'unit' && editValue.trim() === '') value = null;

    dbcStore.updateSignal(selectedMsg.id, signalName, { [field]: value } as Partial<SignalModel>);
    // If we renamed the signal, update selection
    if (field === 'name') {
      selectSignal(selectedMsg.id, String(value));
    }
  }

  function cancelEdit() { editCell = null; }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') { commitEdit(); }
    if (e.key === 'Escape') { cancelEdit(); }
    if (e.key === 'Tab') { commitEdit(); /* natural tab moves focus */ }
  }

  function handleAddSignal() {
    if (!selectedMsg) return;
    const used = new Set(selectedMsg.signals.map((s) => s.name));
    let i = 1;
    while (used.has(`Signal_${i}`)) i++;
    const sig = newSignal();
    sig.name = `Signal_${i}`;
    dbcStore.addSignal(selectedMsg.id, sig);
    selectSignal(selectedMsg.id, sig.name);
  }

  function handleDeleteSignal(sigName: string) {
    if (!selectedMsg) return;
    if (!confirm(`Delete signal "${sigName}"?`)) return;
    dbcStore.deleteSignal(selectedMsg.id, sigName);
    selectedSignalName.set(null);
  }

  // Cell render helper
  function cellValue(sig: SignalModel, field: keyof SignalModel): string {
    if (editCell?.signalName === sig.name && editCell.field === field) return editValue;
    const v = sig[field];
    if (v === null || v === undefined) return '';
    return String(v);
  }

  const COLUMNS: { key: keyof SignalModel; label: string; width: string }[] = [
    { key: 'name',       label: 'Name',       width: '160px' },
    { key: 'start_bit',  label: 'Start',      width: '55px'  },
    { key: 'length',     label: 'Len',        width: '45px'  },
    { key: 'byte_order', label: 'Byte Order', width: '95px'  },
    { key: 'is_unsigned',label: 'Sign',       width: '55px'  },
    { key: 'factor',     label: 'Factor',     width: '65px'  },
    { key: 'offset',     label: 'Offset',     width: '65px'  },
    { key: 'min',        label: 'Min',        width: '65px'  },
    { key: 'max',        label: 'Max',        width: '65px'  },
    { key: 'unit',       label: 'Unit',       width: '65px'  },
  ];
</script>

<div class="signal-table-panel">
  {#if !selectedMsg}
    <div class="empty-state">Select a message in the tree to view its signals.</div>
  {:else}
    <!-- Table header bar -->
    <div class="table-header-bar">
      <span class="msg-title">
        {selectedMsg.name}
        <span class="msg-id-badge">ID: 0x{(selectedMsg.is_extended ? selectedMsg.id & 0x1fffffff : selectedMsg.id).toString(16).toUpperCase()}</span>
        <span class="msg-dlc-badge">DLC: {selectedMsg.dlc}</span>
        <span class="msg-sender-badge">TX: {selectedMsg.sender || '—'}</span>
      </span>
      <button class="add-sig-btn" on:click={handleAddSignal}>+ Add Signal</button>
    </div>

    <!-- Table -->
    <div class="table-scroll">
      <table class="sig-table">
        <thead>
          <tr>
            {#each COLUMNS as col}
              <th style="width:{col.width}">{col.label}</th>
            {/each}
            <th class="col-actions" style="width:40px"></th>
          </tr>
        </thead>
        <tbody>
          {#each selectedMsg.signals as sig (sig.name)}
            {@const isSelected = $selectedSignalName === sig.name}
            <tr
              class:selected={isSelected}
              on:click={() => selectSignal(selectedMsg.id, sig.name)}
            >
              {#each COLUMNS as col}
                <td
                  class="editable-cell"
                  on:dblclick={() => startEdit(sig, col.key)}
                  title="Double-click to edit"
                >
                  {#if editCell?.signalName === sig.name && editCell.field === col.key}
                    {#if col.key === 'byte_order'}
                      <select
                        class="edit-input edit-select"
                        bind:value={editValue}
                        on:blur={commitEdit}
                        on:keydown={onKeydown}
                      >
                        <option value="LittleEndian">LittleEndian</option>
                        <option value="BigEndian">BigEndian</option>
                      </select>
                    {:else if col.key === 'is_unsigned'}
                      <select
                        class="edit-input edit-select"
                        bind:value={editValue}
                        on:blur={commitEdit}
                        on:keydown={onKeydown}
                      >
                        <option value="true">Unsigned</option>
                        <option value="false">Signed</option>
                      </select>
                    {:else}
                      <input
                        class="edit-input"
                        type="text"
                        bind:value={editValue}
                        on:blur={commitEdit}
                        on:keydown={onKeydown}
                      />
                    {/if}
                  {:else}
                    <!-- Display value -->
                    {#if col.key === 'is_unsigned'}
                      <span class="badge" class:badge-unsigned={sig.is_unsigned} class:badge-signed={!sig.is_unsigned}>
                        {sig.is_unsigned ? 'Unsigned' : 'Signed'}
                      </span>
                    {:else if col.key === 'byte_order'}
                      <span class="badge">{sig.byte_order === 'LittleEndian' ? 'Intel' : 'Motorola'}</span>
                    {:else if col.key === 'name'}
                      <span class="sig-name-cell">
                        {#if sig.is_multiplexer}<span class="mux-badge">M</span>{/if}
                        {#if sig.multiplexer_switch_value !== null}<span class="mux-badge">m{sig.multiplexer_switch_value}</span>{/if}
                        {sig.name}
                      </span>
                    {:else}
                      {cellValue(sig, col.key)}
                    {/if}
                  {/if}
                </td>
              {/each}
              <!-- Actions -->
              <td class="col-actions">
                <button
                  class="del-btn"
                  title="Delete signal"
                  on:click|stopPropagation={() => handleDeleteSignal(sig.name)}
                >✕</button>
              </td>
            </tr>
          {/each}

          {#if selectedMsg.signals.length === 0}
            <tr>
              <td colspan={COLUMNS.length + 1} class="no-signals">
                No signals. <button class="link-btn" on:click={handleAddSignal}>Add one</button>.
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .signal-table-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-main);
  }
  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 13px;
  }
  /* Header bar */
  .table-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg-panel);
  }
  .msg-title { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 600; }
  .msg-id-badge, .msg-dlc-badge, .msg-sender-badge {
    font-size: 11px; font-weight: 400;
    background: var(--bg-badge);
    color: var(--text-muted);
    padding: 1px 6px;
    border-radius: 10px;
    font-family: monospace;
  }
  .add-sig-btn {
    font-size: 12px; padding: 4px 10px;
    background: var(--accent); color: #fff;
    border: none; border-radius: 4px; cursor: pointer;
  }
  .add-sig-btn:hover { background: var(--accent-hover); }
  /* Table */
  .table-scroll { flex: 1; overflow: auto; }
  .sig-table {
    width: 100%; border-collapse: collapse;
    font-size: 12px;
  }
  thead { position: sticky; top: 0; z-index: 1; }
  th {
    text-align: left; padding: 5px 8px;
    font-size: 11px; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--text-muted);
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }
  tr { border-bottom: 1px solid var(--border-light); }
  tr:hover td { background: var(--bg-hover); }
  tr.selected td { background: var(--bg-selected-row); }
  td { padding: 4px 8px; vertical-align: middle; }
  .editable-cell { cursor: default; }
  .editable-cell:hover { background: var(--bg-hover-cell) !important; cursor: pointer; }
  /* Inline editor */
  .edit-input {
    width: 100%; box-sizing: border-box;
    border: 1px solid var(--accent); border-radius: 3px;
    padding: 2px 4px; font-size: 12px;
    background: var(--bg-input); color: var(--text);
    outline: none;
  }
  .edit-select { width: 100%; }
  /* Badges */
  .badge {
    font-size: 10px; padding: 1px 5px;
    border-radius: 8px;
    background: var(--bg-badge);
    color: var(--text-muted);
  }
  .badge-unsigned { background: #e6f7ee; color: #276749; }
  .badge-signed   { background: #fff5e6; color: #8a5200; }
  .mux-badge {
    font-size: 9px; font-weight: 700;
    background: #e0e7ff; color: #3730a3;
    padding: 1px 4px; border-radius: 4px;
    margin-right: 4px;
  }
  .sig-name-cell { display: flex; align-items: center; }
  /* Actions col */
  .col-actions { text-align: center; padding: 0 4px; }
  .del-btn {
    background: none; border: none; cursor: pointer;
    padding: 2px 4px; font-size: 11px; color: var(--text-muted);
    border-radius: 3px; opacity: 0;
  }
  tr:hover .del-btn, tr.selected .del-btn { opacity: 1; }
  .del-btn:hover { background: #e53e3e20; color: #e53e3e; }
  .no-signals { color: var(--text-muted); text-align: center; padding: 20px; font-style: italic; }
  .link-btn {
    background: none; border: none; cursor: pointer;
    color: var(--accent); text-decoration: underline; font-size: 12px;
  }
</style>
