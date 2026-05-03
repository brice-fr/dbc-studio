<script lang="ts">
  import { validationIssues, showValidationPanel, selectMessage, selectSignal } from '../stores/ui';
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

  function navigateToIssue(issue: { message_id: number | null; signal_name: string | null }) {
    if (issue.message_id !== null && issue.signal_name !== null) {
      selectSignal(issue.message_id, issue.signal_name);
    } else if (issue.message_id !== null) {
      selectMessage(issue.message_id);
    }
  }
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
          {#if issue.message_id !== null}
            <!-- Navigable: render as interactive button-role div -->
            <div
              class="val-issue navigable"
              class:is-error={issue.severity === 'error'}
              class:is-warning={issue.severity === 'warning'}
              role="button"
              tabindex="0"
              on:click={() => navigateToIssue(issue)}
              on:keydown={(e) => e.key === 'Enter' && navigateToIssue(issue)}
            >
              <span class="issue-icon">{issue.severity === 'error' ? '✖' : '⚠'}</span>
              <span class="issue-msg">{issue.message}</span>
              <span class="issue-loc">
                ID 0x{issue.message_id.toString(16).toUpperCase()}{#if issue.signal_name !== null} · {issue.signal_name}{/if}
              </span>
              <span class="issue-goto" title="Jump to location">→</span>
            </div>
          {:else}
            <!-- Non-navigable: plain div, no tabindex -->
            <div
              class="val-issue"
              class:is-error={issue.severity === 'error'}
              class:is-warning={issue.severity === 'warning'}
            >
              <span class="issue-icon">{issue.severity === 'error' ? '✖' : '⚠'}</span>
              <span class="issue-msg">{issue.message}</span>
            </div>
          {/if}
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
    outline: none;
  }
  .val-issue:last-child { border-bottom: none; }
  .val-issue.navigable  { cursor: pointer; }
  .val-issue.navigable:hover { filter: brightness(0.96); }
  .val-issue.navigable:focus-visible { box-shadow: inset 0 0 0 2px var(--accent); }
  .is-error   { background: #fff5f5; }
  .is-warning { background: #fffbeb; }
  .issue-icon { flex-shrink: 0; font-size: 10px; }
  .is-error   .issue-icon { color: #e53e3e; }
  .is-warning .issue-icon { color: #d97706; }
  .issue-msg  { flex: 1; color: var(--text); }
  .issue-loc  {
    font-size: 10px;
    font-family: monospace;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .issue-goto {
    flex-shrink: 0;
    font-size: 11px;
    color: var(--accent);
    opacity: 0;
    transition: opacity 0.1s;
  }
  .val-issue.navigable:hover .issue-goto,
  .val-issue.navigable:focus-visible .issue-goto { opacity: 1; }
</style>
