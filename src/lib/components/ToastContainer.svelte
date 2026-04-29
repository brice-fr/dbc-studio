<script lang="ts">
  import { toasts } from '../stores/ui';
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div class="toast toast-{toast.type}">
      {toast.message}
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 1000;
    pointer-events: none;
  }
  .toast {
    padding: 8px 14px;
    border-radius: 6px;
    font-size: 13px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.18);
    animation: slidein 0.15s ease;
  }
  .toast-success { background: #276749; color: #fff; }
  .toast-error   { background: #c53030; color: #fff; }
  .toast-info    { background: #2b6cb0; color: #fff; }
  @keyframes slidein {
    from { transform: translateX(20px); opacity: 0; }
    to   { transform: translateX(0);    opacity: 1; }
  }
</style>
