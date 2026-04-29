<script lang="ts">
  import { dbcStore } from '../stores/dbc';
  import { selectedMessageId, selectedSignalName } from '../stores/ui';
  import type { MessageModel, SignalModel } from '../types';

  $: selectedMsg = $dbcStore.messages.find((m) => m.id === $selectedMessageId) ?? null;
  $: selectedSig = selectedMsg?.signals.find((s) => s.name === $selectedSignalName) ?? null;

  // ─── Message editing ──────────────────────────────────────────────────────
  let msgName = '';
  let msgId = '';
  let msgDlc = 8;
  let msgSender = '';
  let msgComment = '';
  let msgExtended = false;

  $: if (selectedMsg && !selectedSig) {
    msgName = selectedMsg.name;
    msgId = (selectedMsg.is_extended ? selectedMsg.id & 0x1fffffff : selectedMsg.id)
      .toString(16)
      .toUpperCase();
    msgDlc = selectedMsg.dlc;
    msgSender = selectedMsg.sender;
    msgComment = selectedMsg.comment ?? '';
    msgExtended = selectedMsg.is_extended;
  }

  function applyMsgChanges() {
    if (!selectedMsg) return;
    const rawId = parseInt(msgId, 16);
    if (isNaN(rawId)) return;
    const id = msgExtended ? rawId | 0x80000000 : rawId;
    dbcStore.updateMessage(selectedMsg.id, {
      id,
      is_extended: msgExtended,
      name: msgName,
      dlc: msgDlc,
      sender: msgSender,
      comment: msgComment || null,
    });
  }

  // ─── Signal editing ───────────────────────────────────────────────────────
  let sigName = '';
  let sigStartBit = 0;
  let sigLength = 8;
  let sigByteOrder = 'LittleEndian';
  let sigUnsigned = true;
  let sigFactor = 1;
  let sigOffset = 0;
  let sigMin = 0;
  let sigMax = 255;
  let sigUnit = '';
  let sigReceivers = '';
  let sigComment = '';
  let sigIsMux = false;
  let sigMuxValue = '';

  $: if (selectedSig) {
    sigName = selectedSig.name;
    sigStartBit = selectedSig.start_bit;
    sigLength = selectedSig.length;
    sigByteOrder = selectedSig.byte_order;
    sigUnsigned = selectedSig.is_unsigned;
    sigFactor = selectedSig.factor;
    sigOffset = selectedSig.offset;
    sigMin = selectedSig.min;
    sigMax = selectedSig.max;
    sigUnit = selectedSig.unit ?? '';
    sigReceivers = selectedSig.receivers.join(', ');
    sigComment = selectedSig.comment ?? '';
    sigIsMux = selectedSig.is_multiplexer;
    sigMuxValue = selectedSig.multiplexer_switch_value?.toString() ?? '';
  }

  function applySigChanges() {
    if (!selectedMsg || !selectedSig) return;
    const patch: Partial<SignalModel> = {
      name: sigName,
      start_bit: sigStartBit,
      length: sigLength,
      byte_order: sigByteOrder,
      is_unsigned: sigUnsigned,
      factor: sigFactor,
      offset: sigOffset,
      min: sigMin,
      max: sigMax,
      unit: sigUnit || null,
      receivers: sigReceivers.split(',').map((r) => r.trim()).filter(Boolean),
      comment: sigComment || null,
      is_multiplexer: sigIsMux,
      multiplexer_switch_value: sigMuxValue !== '' ? parseInt(sigMuxValue) : null,
    };
    dbcStore.updateSignal(selectedMsg.id, selectedSig.name, patch);
    if (sigName !== selectedSig.name) {
      selectedSignalName.set(sigName);
    }
  }

  // Physical value preview
  $: physMin = sigMin * sigFactor + sigOffset;
  $: physMax = sigMax * sigFactor + sigOffset;
</script>

<div class="props-panel">
  {#if selectedSig && selectedMsg}
    <!-- Signal properties -->
    <div class="panel-header">Signal: {selectedSig.name}</div>
    <form class="props-form" on:submit|preventDefault={applySigChanges}>

      <fieldset>
        <legend>Identification</legend>
        <label>Name <input bind:value={sigName} /></label>
        <label>
          Byte Order
          <select bind:value={sigByteOrder}>
            <option value="LittleEndian">Intel (Little Endian)</option>
            <option value="BigEndian">Motorola (Big Endian)</option>
          </select>
        </label>
        <label>
          Value Type
          <select bind:value={sigUnsigned}>
            <option value={true}>Unsigned</option>
            <option value={false}>Signed</option>
          </select>
        </label>
      </fieldset>

      <fieldset>
        <legend>Bit Position</legend>
        <label>Start Bit <input type="number" bind:value={sigStartBit} min="0" /></label>
        <label>Length (bits) <input type="number" bind:value={sigLength} min="1" max="64" /></label>
      </fieldset>

      <fieldset>
        <legend>Scaling</legend>
        <label>Factor <input type="number" bind:value={sigFactor} step="any" /></label>
        <label>Offset <input type="number" bind:value={sigOffset} step="any" /></label>
        <label>Min <input type="number" bind:value={sigMin} step="any" /></label>
        <label>Max <input type="number" bind:value={sigMax} step="any" /></label>
        <label>Unit <input bind:value={sigUnit} placeholder="e.g. rpm, km/h" /></label>
        <div class="phys-preview">
          Physical: [{physMin.toPrecision(4)} … {physMax.toPrecision(4)}] {sigUnit}
        </div>
      </fieldset>

      <fieldset>
        <legend>Multiplexing</legend>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={sigIsMux} /> Multiplexer switch (M)
        </label>
        <label>Mux value (m&lt;N&gt;) <input bind:value={sigMuxValue} placeholder="none" /></label>
      </fieldset>

      <fieldset>
        <legend>Other</legend>
        <label>Receivers <input bind:value={sigReceivers} placeholder="ECM, TCM (comma-separated)" /></label>
        <label>Comment <textarea bind:value={sigComment} rows="2"></textarea></label>
      </fieldset>

      <button type="submit" class="apply-btn">Apply Changes</button>
    </form>

  {:else if selectedMsg}
    <!-- Message properties -->
    <div class="panel-header">Message: {selectedMsg.name}</div>
    <form class="props-form" on:submit|preventDefault={applyMsgChanges}>

      <fieldset>
        <legend>Identification</legend>
        <label>Name <input bind:value={msgName} /></label>
        <label class="id-row">
          CAN ID (hex)
          <div class="id-input-wrap">
            <span class="hex-prefix">0x</span>
            <input class="hex-input" bind:value={msgId} pattern="[0-9A-Fa-f]+" />
          </div>
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={msgExtended} /> Extended (29-bit)
        </label>
        <label>
          DLC (bytes)
          <input type="number" bind:value={msgDlc} min="0" max="64" />
        </label>
        <label>Transmitter <input bind:value={msgSender} placeholder="e.g. ECM" /></label>
      </fieldset>

      <fieldset>
        <legend>Other</legend>
        <label>Comment <textarea bind:value={msgComment} rows="3"></textarea></label>
      </fieldset>

      <button type="submit" class="apply-btn">Apply Changes</button>
    </form>

  {:else}
    <div class="empty-props">Select a message or signal to view properties.</div>
  {/if}
</div>

<style>
  .props-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-panel);
    border-left: 1px solid var(--border);
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .props-form {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  fieldset {
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  legend {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 0 4px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
    color: var(--text-muted);
  }
  label input, label select, label textarea {
    font-size: 12px;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input);
    color: var(--text);
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }
  label input:focus, label select:focus, label textarea:focus {
    border-color: var(--accent);
  }
  label textarea { resize: vertical; min-height: 40px; }
  .checkbox-label { flex-direction: row; align-items: center; gap: 8px; }
  .checkbox-label input { width: auto; }
  .id-row .id-input-wrap {
    display: flex; align-items: center;
    border: 1px solid var(--border); border-radius: 4px;
    overflow: hidden;
    background: var(--bg-input);
  }
  .hex-prefix { padding: 4px 6px; font-size: 12px; color: var(--text-muted); background: var(--bg-badge); }
  .hex-input { border: none !important; border-radius: 0 !important; flex: 1; font-family: monospace; }
  .phys-preview {
    font-size: 11px;
    color: var(--accent);
    background: var(--bg-badge);
    border-radius: 4px;
    padding: 3px 6px;
    font-family: monospace;
  }
  .apply-btn {
    margin-top: 4px;
    padding: 6px 12px;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    align-self: flex-start;
  }
  .apply-btn:hover { background: var(--accent-hover); }
  .empty-props {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 12px;
    text-align: center;
    padding: 16px;
  }
</style>
