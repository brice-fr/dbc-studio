<script lang="ts">
  import { validationIssues, showValidationPanel } from '../stores/ui';
  import { dbcStore } from '../stores/dbc';
  import { validateDbc } from '../api';

  export let running = false;

  export async function runValidation() {
    running = true;
    try {
      const issues = await validateDbc($dbcStore);
      validationIssues.set(issues);
      showValidationPanel.set(true);
    } catch (e) {
      console.error('Validation error', e);
    } finally {
      running = false;
    }
  }

  $: errorCount   = $validationIssues.filter((i) => i.severity === 'error').length;
  $: warningCount = $validationIssues.filter((i) => i.severity === 'warning').length;
</script>

{#if $showValidationPanel}
  <div class="val-panel">
    <div class="val-header">
      <span class="val-title">
        Validation
        {#if errorCount > 0}
          <span class="badge badge-error">{errorCount} error{errorCount !== 1 ? 's' : ''}</span>
        {/if}
        {#if warningCount > 0}
          <span class="badge badge-warning">{warningCount} warning{warningCount !== 1 ? 's' : ''}</span>
        {/if}
        {#if errorCount === 0 && warningCount === 0}
          <span class="badge badge-ok">✓ No issues</span>
        {/if}
      </span>
      <button class="close-btn" on:click={() => showValidationPanel.set(false)}>✕</button>
    </div>

    {#if $validationIssues.length === 0}
      <div class="val-empty">All checks passed.</div>
    {:else}
      <div class="val-list">
        {#each $validationIssues as issue}
          <div class="val-issue" class:is-error={issue.severity === 'error'} class:is-warning={issue.severity === 'warning'}>
            <span class="issue-icon">{issue.severity === 'error' ? '✖' : '⚠'}</span>
            <span class="issue-msg">{issue.message}</span>
            {#if issue.message_id !== null || issue.signal_name !== null}
              <span class="issue-loc">
                {#if issue.message_id !== null}ID 0x{issue.message_id.toString(16).toUpperCase()}{/if}{#if issue.message_id !== null && issue.signal_name !== null} · {/if}{#if issue.signal_name !== null}{issue.signal_name}{/if}
              </span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .val-panel {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    background: var(--bg-panel);
    display: flex;
    flex-direction: column;
    max-height: 200px;
  }
  .val-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 10px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .val-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }
  .badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    font-weight: 600;
    text-transform: none;
    letter-spacing: 0;
  }
  .badge-error   { background: #fee2e2; color: #b91c1c; }
  .badge-warning { background: #fef3c7; color: #92400e; }
  .badge-ok      { background: #d1fae5; color: #065f46; }
  .close-btn {
    background: none; border: none; cursor: pointer;
    font-size: 11px; color: var(--text-muted); padding: 2px 4px; border-radius: 3px;
  }
  .close-btn:hover { background: var(--bg-hover); }
  .val-empty {
    padding: 10px 12px;
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
  }
  .val-list {
    overflow-y: auto;
    flex: 1;
  }
  .val-issue {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 4px 10px;
    font-size: 12px;
    border-bottom: 1px solid var(--border-light);
  }
  .val-issue:last-child { border-bottom: none; }
  .is-error  { background: #fff5f5; }
  .is-warning { background: #fffbeb; }
  .issue-icon {
    flex-shrink: 0;
    font-size: 10px;
  }
  .is-error   .issue-icon { color: #e53e3e; }
  .is-warning .issue-icon { color: #d97706; }
  .issue-msg  { flex: 1; color: var(--text); }
  .issue-loc  {
    font-size: 10px;
    font-family: monospace;
    color: var(--text-muted);
    flex-shrink: 0;
  }
</style>
