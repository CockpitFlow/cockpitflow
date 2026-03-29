<script lang="ts" module>
  type ToastType = 'success' | 'error' | 'info' | 'warning';
  interface ToastItem { id: number; message: string; type: ToastType; }

  let toasts: ToastItem[] = $state([]);
  let nextId = 0;

  export function toast(message: string, type: ToastType = 'info', duration = 3000) {
    const id = nextId++;
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, duration);
  }
</script>

<script lang="ts">
</script>

{#if toasts.length > 0}
  <div class="toast-container">
    {#each toasts as t (t.id)}
      <div class="toast toast-{t.type}" role="alert">
        <span class="toast-icon">
          {#if t.type === 'success'}&#x2713;{:else if t.type === 'error'}&#x2717;{:else if t.type === 'warning'}&#x26A0;{:else}&#x2139;{/if}
        </span>
        <span class="toast-msg">{t.message}</span>
        <button class="toast-close" onclick={() => toasts = toasts.filter(x => x.id !== t.id)}>&times;</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 9999;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    max-width: 380px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 500;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    animation: toastIn 0.25s ease-out;
    backdrop-filter: blur(12px);
  }

  .toast-success { background: rgba(52, 211, 153, 0.15); border: 1px solid rgba(52, 211, 153, 0.3); color: #34d399; }
  .toast-error { background: rgba(248, 113, 113, 0.15); border: 1px solid rgba(248, 113, 113, 0.3); color: #f87171; }
  .toast-warning { background: rgba(251, 191, 36, 0.15); border: 1px solid rgba(251, 191, 36, 0.3); color: #fbbf24; }
  .toast-info { background: rgba(79, 143, 234, 0.15); border: 1px solid rgba(79, 143, 234, 0.3); color: #4f8fea; }

  .toast-icon { font-size: 14px; flex-shrink: 0; }
  .toast-msg { flex: 1; color: var(--text, #e8ecf4); }
  .toast-close { background: none; border: none; color: inherit; cursor: pointer; font-size: 16px; opacity: 0.5; padding: 0 2px; }
  .toast-close:hover { opacity: 1; }

  @keyframes toastIn {
    from { opacity: 0; transform: translateY(10px) scale(0.95); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
</style>
