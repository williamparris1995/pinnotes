<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';
  let items = $state<Note[]>([]);
  onMount(async () => (items = await invoke<Note[]>('list_completed')));
</script>

<main>
  <h2>已完成</h2>
  {#if items.length === 0}
    <p>暂无已完成项</p>
  {:else}
    <ul>
      {#each items as it}
        <li>
          <span class="dot" style="background:var(--c-{it.color})"></span>
          <span class="txt">{it.content}</span>
          <button onclick={() => invoke('reactivate', { id: it.id })}>重新激活</button>
          <button onclick={() => invoke('copy_note', { id: it.id })}>复制</button>
          <button onclick={() => invoke('edit_note', { id: it.id, content: prompt('编辑', it.content) ?? it.content })}>编辑</button>
          <button onclick={() => invoke('delete_note', { id: it.id })}>删除</button>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  main { font-family: var(--font); padding: 16px; }
  li { list-style: none; display: flex; align-items: center; gap: 8px; padding: 6px 0; }
  .dot { width: 12px; height: 12px; border-radius: 50%; display: inline-block; }
  .txt { flex: 1; }
</style>
