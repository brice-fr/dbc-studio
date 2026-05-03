<!--
  NodePicker — single or multi-select from the DBC node list.

  Single mode  (multiple=false): bind:value  → string
  Multi  mode  (multiple=true):  bind:values → string[]
  Both modes offer an inline "＋ New node" flow that adds the node to the store.
-->
<script lang="ts">
  import { dbcStore } from '../stores/dbc';
  import { showToast } from '../stores/ui';

  /** Single-select mode: currently selected node name (or '' for none). */
  export let value  = '';
  /** Multi-select mode: currently selected node names. */
  export let values: string[] = [];
  /** When true, renders checkboxes; when false, renders radio-like single select. */
  export let multiple = false;

  // ─── New node input state ────────────────────────────────────────────────
  let addingNew = false;
  let newName   = '';

  function startAdd() { addingNew = true; newName = ''; }

  function commitAdd() {
    const name = newName.trim();
    if (!name) { addingNew = false; return; }
    if ($dbcStore.nodes.some((n) => n.name === name)) {
      showToast('error', `Node "${name}" already exists`);
      return;
    }
    dbcStore.addNode({ name, comment: null });
    showToast('success', `Created node "${name}"`);
    // Auto-select the new node
    if (multiple) {
      if (!values.includes(name)) values = [...values, name];
    } else {
      value = name;
    }
    addingNew = false;
    newName   = '';
  }

  function cancelAdd() { addingNew = false; newName = ''; }

  // ─── Toggle helpers ──────────────────────────────────────────────────────
  function toggleMulti(name: string) {
    if (values.includes(name)) {
      values = values.filter((v) => v !== name);
    } else {
      values = [...values, name];
    }
  }

  function focusInput(el: HTMLElement) {
    el.focus();
    (el as HTMLInputElement).select?.();
    return {};
  }
</script>

<div class="node-picker">
  {#if multiple}
    <!-- ── Multi-select: checkboxes ──────────────────────────────────────── -->
    {#if $dbcStore.nodes.length === 0 && !addingNew}
      <span class="picker-empty">No nodes defined.</span>
    {/if}
    {#each $dbcStore.nodes as node (node.name)}
      <label class="picker-row">
        <input
          type="checkbox"
          checked={values.includes(node.name)}
          on:change={() => toggleMulti(node.name)}
        />
        <span class="picker-name">{node.name}</span>
      </label>
    {/each}

  {:else}
    <!-- ── Single-select: radio-style rows ───────────────────────────────── -->
    <label class="picker-row">
      <input type="radio" bind:group={value} value="" />
      <span class="picker-name picker-none">None</span>
    </label>
    {#each $dbcStore.nodes as node (node.name)}
      <label class="picker-row">
        <input type="radio" bind:group={value} value={node.name} />
        <span class="picker-name">{node.name}</span>
      </label>
    {/each}
  {/if}

  <!-- ── Add new node inline ───────────────────────────────────────────── -->
  {#if addingNew}
    <div class="picker-add-row">
      <span class="picker-add-icon">⬡</span>
      <input
        class="picker-new-input"
        placeholder="New node name"
        bind:value={newName}
        on:keydown={(e) => {
          if (e.key === 'Enter')  commitAdd();
          if (e.key === 'Escape') cancelAdd();
        }}
        on:blur={commitAdd}
        use:focusInput
      />
    </div>
  {:else}
    <button type="button" class="picker-add-btn" on:click={startAdd}>＋ New node…</button>
  {/if}
</div>

<style>
  .node-picker {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 2px 0;
  }
  .picker-empty {
    font-size: 11px;
    color: var(--text-muted);
    font-style: italic;
    padding: 2px 0;
  }
  .picker-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    color: var(--text);
  }
  .picker-row:hover { background: var(--bg-hover); }
  .picker-row input[type="checkbox"],
  .picker-row input[type="radio"] {
    width: auto;
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .picker-name { flex: 1; }
  .picker-none { color: var(--text-muted); font-style: italic; }
  /* Add new row */
  .picker-add-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 4px;
  }
  .picker-add-icon { font-size: 10px; color: var(--text-muted); flex-shrink: 0; }
  .picker-new-input {
    flex: 1;
    height: 22px;
    padding: 0 5px;
    font-size: 12px;
    border: 1px solid var(--accent);
    border-radius: 3px;
    background: var(--bg-input);
    color: var(--text);
    outline: none;
  }
  .picker-add-btn {
    align-self: flex-start;
    font-size: 11px;
    padding: 3px 8px;
    background: none;
    border: 1px dashed var(--border);
    border-radius: 4px;
    color: var(--text-muted);
    cursor: pointer;
    margin-top: 2px;
  }
  .picker-add-btn:hover { border-color: var(--accent); color: var(--accent); background: var(--bg-hover); }
</style>
