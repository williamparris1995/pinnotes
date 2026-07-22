<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';

  let { id }: { id: string } = $props();
  let note = $state<Note | null>(null);
  let editing = $state(false);
  let draft = $state('');
  let loadError = $state(false);

  // On failure, show a fallback rather than leaving a blank, always-on-top
  // transparent window stuck on screen.
  onMount(async () => {
    try {
      note = await invoke<Note>('get_note', { id });
    } catch {
      loadError = true;
    }
  });

  function startEdit() { if (note) { draft = note.content; editing = true; } }
  async function commit() {
    if (note && draft !== note.content) await invoke('edit_note', { id, content: draft });
    if (note) note.content = draft;
    editing = false;
  }
  const colorVar = $derived(note ? `var(--c-${note.color})` : 'transparent');
</script>

{#if loadError}
  <div class="note error">无法加载便签</div>
{:else if note}
  <div class="note" style="background:{colorVar}">
    <div class="grip" data-tauri-drag-region></div>
    {#if editing}
      <textarea bind:value={draft} onfocusout={commit}></textarea>
    {:else}
      <p ondblclick={startEdit}>{note.content || '（空）'}</p>
    {/if}
    <div class="actions">
      <button onclick={() => invoke('hide_note', { id })}>隐藏</button>
      <button onclick={() => invoke('complete_note', { id })}>✓ 完成</button>
    </div>
  </div>
{/if}

<style>
  .note { width: 240px; padding: 10px; border-radius: var(--radius);
          box-shadow: 0 6px 16px rgba(0,0,0,.2); }
  .note.error { background: #eee; color: #b00; }
  .grip { width: 40px; height: 4px; margin: 0 auto 8px; background: rgba(0,0,0,.25); border-radius: 2px; }
  textarea { width: 100%; min-height: 60px; }
  .actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
</style>
