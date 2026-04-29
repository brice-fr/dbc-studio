<script lang="ts">
  import Toolbar from '$lib/components/Toolbar.svelte';
  import TreePanel from '$lib/components/TreePanel.svelte';
  import SignalTable from '$lib/components/SignalTable.svelte';
  import PropertiesPanel from '$lib/components/PropertiesPanel.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
  import { treePanelWidth, propertiesPanelWidth } from '$lib/stores/ui';
  import { isDirty, currentFilePath } from '$lib/stores/dbc';

  let validationPanel: ValidationPanel;

  // Update window title
  $: document.title = `DBC Studio${$currentFilePath ? ' — ' + $currentFilePath.split('/').pop() : ''}${$isDirty ? ' ●' : ''}`;

  // Resizable panels
  let draggingLeft = false;
  let draggingRight = false;
  let dragStartX = 0;
  let dragStartWidth = 0;

  function startDragLeft(e: MouseEvent) {
    draggingLeft = true; dragStartX = e.clientX; dragStartWidth = $treePanelWidth; e.preventDefault();
  }
  function startDragRight(e: MouseEvent) {
    draggingRight = true; dragStartX = e.clientX; dragStartWidth = $propertiesPanelWidth; e.preventDefault();
  }
  function onMouseMove(e: MouseEvent) {
    if (draggingLeft)  treePanelWidth.set(Math.max(160, Math.min(500, dragStartWidth + e.clientX - dragStartX)));
    if (draggingRight) propertiesPanelWidth.set(Math.max(200, Math.min(500, dragStartWidth - (e.clientX - dragStartX))));
  }
  function stopDrag() { draggingLeft = false; draggingRight = false; }
</script>

<svelte:window on:mousemove={onMouseMove} on:mouseup={stopDrag} />

<div class="app-shell">
  <Toolbar on:validate={() => validationPanel?.runValidation()} />
  <div class="workspace">
    <div class="tree-pane" style="width:{$treePanelWidth}px; min-width:{$treePanelWidth}px;">
      <TreePanel />
    </div>
    <div class="resize-handle left-handle" on:mousedown={startDragLeft} role="none"></div>
    <div class="center-pane">
      <SignalTable />
    </div>
    <div class="resize-handle right-handle" on:mousedown={startDragRight} role="none"></div>
    <div class="props-pane" style="width:{$propertiesPanelWidth}px; min-width:{$propertiesPanelWidth}px;">
      <PropertiesPanel />
    </div>
  </div>
  <ValidationPanel bind:this={validationPanel} />
</div>

<ToastContainer />

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(:root) {
    --bg-main:        #ffffff;
    --bg-panel:       #f7f8fa;
    --bg-toolbar:     #f0f1f3;
    --bg-hover:       #e8eaed;
    --bg-hover-cell:  #eef0f3;
    --bg-selected:    #1a56db;
    --bg-selected-row:#eff6ff;
    --bg-badge:       #e8eaed;
    --bg-input:       #ffffff;
    --border:         #d1d5db;
    --border-light:   #e5e7eb;
    --text:           #111827;
    --text-muted:     #6b7280;
    --text-selected:  #ffffff;
    --accent:         #1a56db;
    --accent-hover:   #1347c2;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 13px;
    color: var(--text);
  }
  :global(body) { background: var(--bg-main); overflow: hidden; height: 100vh; }
  .app-shell { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
  .workspace  { display: flex; flex: 1; overflow: hidden; }
  .tree-pane  { display: flex; flex-direction: column; overflow: hidden; flex-shrink: 0; }
  .center-pane{ flex: 1; overflow: hidden; display: flex; flex-direction: column; }
  .props-pane { display: flex; flex-direction: column; overflow: hidden; flex-shrink: 0; }
  .resize-handle {
    width: 4px; background: var(--border);
    cursor: col-resize; flex-shrink: 0;
    transition: background 0.15s;
  }
  .resize-handle:hover { background: var(--accent); }
</style>
