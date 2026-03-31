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
  <div class="fixed bottom-10 left-1/2 -translate-x-1/2 z-[9999] flex flex-col gap-2 items-center">
    {#each toasts as t (t.id)}
      <div class="px-5 py-2.5 rounded-lg text-sm font-semibold shadow-lg animate-[toastIn_0.2s_ease-out]
        {t.type === 'success' ? 'bg-[var(--color-green)] text-white' :
         t.type === 'error' ? 'bg-[var(--color-red)] text-white' :
         t.type === 'warning' ? 'bg-[var(--color-yellow)] text-black' :
         'bg-[var(--color-accent)] text-white'}" role="alert">
        {t.message}
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes toastIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
