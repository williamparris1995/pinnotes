<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';

  // Prototype pinnotes-8c76 · Section 02 CompletedWindow: each row is a color
  // swatch + truncated text + completed time + 3 icon actions
  // (重新激活 / 复制 / 删除). After reactivate/delete the list re-fetches so
  // the row disappears; copy leaves the original and the backend opens a new
  // active note window.
  let items = $state<Note[]>([]);
  onMount(async () => (items = await invoke<Note[]>('list_completed')));

  // Match the prototype's "今天 HH:MM" / "昨天 HH:MM" timestamps.
  function fmtTime(iso: string | null): string {
    if (!iso) return '';
    const d = new Date(iso);
    if (isNaN(d.getTime())) return '';
    const now = new Date();
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    if (d.toDateString() === now.toDateString()) return `今天 ${hh}:${mm}`;
    const yest = new Date(now);
    yest.setDate(now.getDate() - 1);
    if (d.toDateString() === yest.toDateString()) return `昨天 ${hh}:${mm}`;
    return `${d.getMonth() + 1}月${d.getDate()}日 ${hh}:${mm}`;
  }

  // Only reactivate / copy / delete are exposed (edit was removed per request).
  async function reactivate(it: Note) {
    await invoke('reactivate', { id: it.id });
    items = await invoke<Note[]>('list_completed');
  }
  async function copyNote(it: Note) {
    await invoke('copy_note', { id: it.id });
  }
  async function deleteNote(it: Note) {
    await invoke('delete_note', { id: it.id });
    items = await invoke<Note[]>('list_completed');
  }
</script>

<main>
  {#each items as it (it.id)}
    <div class="done-item">
      <span class="done-swatch swatch-{it.color}"></span>
      <span class="done-text" title={it.content}>{it.content}</span>
      <span class="done-time">{fmtTime(it.completed_at)}</span>
      <div class="done-actions">
        <button type="button" class="done-act" title="重新激活" aria-label="重新激活" onclick={() => reactivate(it)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 7 21 7 21 3"/><path d="M21 7l-4 4"/><path d="M3 17a9 9 0 0 0 15 6"/></svg>
        </button>
        <button type="button" class="done-act" title="复制" aria-label="复制" onclick={() => copyNote(it)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></svg>
        </button>
        <button type="button" class="done-act danger" title="删除" aria-label="删除" onclick={() => deleteNote(it)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        </button>
      </div>
    </div>
  {/each}
  <div class="empty-state">
    <div class="es-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 3v4a1 1 0 0 0 1 1h4"/><path d="M17 21H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7l5 5v11a2 2 0 0 1-2 2z"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="9" y1="17" x2="13" y2="17"/></svg>
    </div>
    <div>暂无更多已完成项 · 历史记录保留 30 天</div>
  </div>
</main>

<style>
  /* Ported from OD prototype (pinnotes-8c76) — exact values for the completed
     window body. The auxiliary windows use native OS chrome, so only the list
     rows + footer empty-state are rendered. */
  main {
    --canvas-soft: #fafafb;
    --ink: #1a1a20;
    --ink-2: #5a5a66;
    --ink-3: #8a8a96;
    --line: #e4e4ea;
    --danger: #c0392b;
    font-family: var(--font);
    background: #fcfcfe;
    min-height: 100%;
    box-sizing: border-box;
    padding: 6px 0 4px;
  }

  .done-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    transition: background 0.12s;
  }
  .done-item:hover { background: rgba(0, 0, 0, 0.025); }
  .done-swatch {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    flex-shrink: 0;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.06);
  }
  /* swatch-* are referenced dynamically (swatch-{color}), so declare them
     :global to avoid unused-selector warnings while keeping the prototype's
     exact class names. Each aux window renders only this component, so the
     global leak is contained. */
  :global(.swatch-yellow) { background: #ffe078; }
  :global(.swatch-pink) { background: #ffc0c8; }
  :global(.swatch-blue) { background: #a0cdff; }
  :global(.swatch-green) { background: #aae6a8; }
  .done-text {
    flex: 1;
    font-size: 13px;
    color: var(--ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .done-time {
    font-size: 11px;
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .done-actions {
    display: flex;
    gap: 1px;
    margin-left: 8px;
    opacity: 0.55;
    transition: opacity 0.12s;
  }
  .done-item:hover .done-actions { opacity: 1; }
  .done-act {
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink-2);
    cursor: pointer;
    transition: all 0.12s;
  }
  .done-act:hover { background: rgba(0, 0, 0, 0.06); color: var(--ink); }
  .done-act.danger:hover { background: #ffeaea; color: var(--danger); }
  .done-act svg { width: 13px; height: 13px; stroke-width: 2; }

  .empty-state {
    padding: 16px 16px 18px;
    text-align: center;
    font-size: 12px;
    color: var(--ink-3);
    border-top: 1px dashed var(--line);
    margin-top: 4px;
    line-height: 1.7;
  }
  .empty-state .es-icon {
    display: inline-flex;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--canvas-soft);
    border: 1px solid var(--line);
    align-items: center;
    justify-content: center;
    color: var(--ink-3);
    margin-bottom: 6px;
  }
  .empty-state .es-icon svg { width: 13px; height: 13px; }
</style>
