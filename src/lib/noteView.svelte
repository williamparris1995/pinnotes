<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let { id }: { id: string } = $props();
  let note = $state<Note | null>(null);
  let draft = $state('');
  let loadError = $state(false);
  // #6: pending completion shows an undo toast instead of closing immediately.
  let pendingComplete = $state(false);
  let completeTimer: ReturnType<typeof setTimeout> | null = null;

  // #3/#4: color presets (match the .note-yellow/pink/blue/green CSS) and the
  // two size presets (普通 = 240×170 default, 大号 = 360×260).
  const COLORS = [
    { id: 'yellow', label: '黄' },
    { id: 'pink', label: '粉' },
    { id: 'blue', label: '蓝' },
    { id: 'green', label: '绿' },
  ] as const;
  const SIZE_NORMAL = { w: 240, h: 170 };
  const SIZE_LARGE = { w: 360, h: 260 };
  // #2: per-note hide duration cycled in the toolbar; takes effect on next 隐藏.
  const SNOOZE_OPTS = [1, 2, 5, 10, 30, 60] as const;
  let isLarge = $derived(note ? note.w >= 340 : false);

  let taRef = $state<HTMLTextAreaElement | null>(null);

  // On failure, show a fallback rather than leaving a blank, always-on-top
  // transparent window stuck on screen.
  onMount(async () => {
    try {
      const n = await invoke<Note>('get_note', { id });
      note = n;
      // The body is an always-editable textarea; seed it with the saved text.
      draft = n.content;
    } catch {
      loadError = true;
    }
  });

  // Focus a fresh/empty note so the user can start typing immediately.
  $effect(() => {
    if (taRef && note && note.content === '') {
      taRef.focus();
      const len = taRef.value.length;
      taRef.selectionStart = len;
      taRef.selectionEnd = len;
    }
  });

  // Save on blur — the textarea is always present, so there is no edit-mode
  // toggle to exit; clicking colour/size/hidden/complete simply blurs it and
  // persists any change.
  async function commit() {
    if (note && draft !== note.content) {
      await invoke('edit_note', { id, content: draft });
      note.content = draft;
    }
  }

  // Debounced autosave on input. Always-on-top windows blur unpredictably, so
  // focusout (commit) alone can miss a save — which left notes blank/empty on
  // reopen. Save ~500ms after typing stops as well.
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  function onInput() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      if (note && draft !== note.content) {
        await invoke('edit_note', { id, content: draft });
        note.content = draft;
      }
    }, 500);
  }

  // #3: change color live — persist + update reactively so the note-{color}
  // class re-applies the background immediately.
  function setColor(color: string) {
    if (!note || note.color === color) return;
    invoke('set_color', { id, color });
    note.color = color;
  }

  // #4: toggle between 普通 and 大号. The OS window resizes; the .note
  // (100% width/height) fills it.
  function toggleSize() {
    if (!note) return;
    const target = isLarge ? SIZE_NORMAL : SIZE_LARGE;
    invoke('set_size', { id, w: target.w, h: target.h });
    note.w = target.w;
    note.h = target.h;
  }

  // #2: cycle through the preset hide durations (wraps around); persists and
  // updates reactively so the button label refreshes immediately.
  function cycleSnooze() {
    if (!note) return;
    const idx = SNOOZE_OPTS.indexOf(note.snooze_minutes as (typeof SNOOZE_OPTS)[number]);
    const next = SNOOZE_OPTS[(idx + 1) % SNOOZE_OPTS.length];
    invoke('set_snooze', { id, minutes: next });
    note.snooze_minutes = next;
  }

  // #6: defer the real complete_note by 5s so the user can undo a misclick;
  // the note stays put until the timer fires.
  function onComplete() {
    if (!note || pendingComplete) return;
    pendingComplete = true;
    completeTimer = setTimeout(() => {
      invoke('complete_note', { id });
    }, 5000);
  }
  function undoComplete() {
    if (completeTimer) { clearTimeout(completeTimer); completeTimer = null; }
    pendingComplete = false;
  }
</script>

{#if loadError}
  <div class="note note-error">无法加载便签</div>
{:else if note}
  <article class="note note-{note.color}">
    <div
      class="note-grip"
      role="button"
      tabindex="0"
      aria-label="拖动"
      onpointerdown={() => getCurrentWindow().startDragging()}
    ></div>
    <div class="note-toolbar">
      <div class="color-dots">
        {#each COLORS as c (c.id)}
          <button
            type="button"
            class="color-dot color-dot-{c.id}"
            class:active={note.color === c.id}
            aria-label="颜色：{c.label}"
            aria-pressed={note.color === c.id}
            onclick={() => setColor(c.id)}
          ></button>
        {/each}
      </div>
      <div class="toolbar-tools">
        <button type="button" class="size-btn" title="隐藏时长（分钟）" onclick={cycleSnooze}>
          {note.snooze_minutes}分
        </button>
        <button type="button" class="size-btn" onclick={toggleSize}>
          {isLarge ? '小' : '大'}
        </button>
      </div>
    </div>
    <div class="note-body">
      <textarea
        bind:value={draft}
        bind:this={taRef}
        oninput={onInput}
        onfocusout={commit}
        placeholder="输入提醒内容…"
      ></textarea>
    </div>
    {#if pendingComplete}
      <div class="note-toast" role="status">
        <span>已完成</span>
        <button type="button" class="toast-undo" onclick={undoComplete}>撤销</button>
      </div>
    {/if}
    <div class="note-actions">
      <button class="note-btn" onclick={() => invoke('hide_note', { id })}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15 14"/></svg>
        隐藏
      </button>
      <button class="note-btn note-btn--done" onclick={onComplete}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        完成
      </button>
    </div>
  </article>
{/if}

<style>
  /* Ported from the OD prototype (project pinnotes-8c76) so the live note
     matches the design: translucent pastel + backdrop blur, grip header,
     bold body, pill action buttons. The card fills its window (the window's
     w/h defines the note size; default 240x170 = the prototype note). */
  :global(html, body, #app) { height: 100%; margin: 0; }

  .note {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    border-radius: 10px;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.2), 0 3px 8px rgba(0, 0, 0, 0.12);
    display: flex;
    flex-direction: column;
    backdrop-filter: blur(8px) saturate(130%);
    -webkit-backdrop-filter: blur(8px) saturate(130%);
    border: 1px solid rgba(255, 255, 255, 0.45);
    overflow: hidden;
    position: relative;
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
  }
  .note-error { display: block; padding: 16px; background: #eee; color: #b00; }

  .note-yellow { background: rgba(255, 230, 120, 0.82); }
  .note-pink   { background: rgba(255, 192, 200, 0.82); }
  .note-blue   { background: rgba(160, 205, 255, 0.82); }
  .note-green  { background: rgba(170, 230, 168, 0.82); }

  .note-grip {
    flex: 0 0 auto;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 16px;
    padding-top: 7px;
    cursor: grab;
  }
  .note-grip::before {
    content: '';
    width: 26px;
    height: 3px;
    border-radius: 2px;
    background: rgba(0, 0, 0, 0.16);
  }

  .note-toolbar {
    flex: 0 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 10px 2px;
  }
  .color-dots { display: flex; gap: 4px; }
  .toolbar-tools { display: flex; gap: 4px; }
  .color-dot {
    width: 14px;
    height: 14px;
    padding: 0;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.18);
    cursor: pointer;
  }
  .color-dot-yellow { background: #ffe678; }
  .color-dot-pink   { background: #ffc0c8; }
  .color-dot-blue   { background: #a0cdff; }
  .color-dot-green  { background: #aae6a8; }
  .color-dot:hover { transform: scale(1.12); }
  .color-dot.active { box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.92); }
  .size-btn {
    padding: 2px 7px;
    border-radius: 4px;
    border: none;
    background: rgba(255, 255, 255, 0.5);
    color: rgba(15, 15, 25, 0.72);
    font-size: 10px;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.3;
    cursor: pointer;
  }
  .size-btn:hover { background: rgba(255, 255, 255, 0.85); }

  .note-body {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
  }
  .note-body textarea {
    flex: 1 1 auto;
    width: 100%;
    box-sizing: border-box;
    font-family: inherit;
    font-size: 15px;
    font-weight: 600;
    color: rgba(15, 15, 25, 0.86);
    line-height: 1.4;
    border: none;
    border-radius: 0;
    padding: 2px 16px 4px;
    resize: none;
    outline: none;
    background: transparent;
    white-space: pre-wrap;
    word-break: break-word;
    scrollbar-width: none;
  }
  .note-body textarea::-webkit-scrollbar { display: none; }
  .note-body textarea::placeholder { color: rgba(15, 15, 25, 0.4); font-weight: 500; }

  .note-toast {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin: 0 8px 4px;
    padding: 5px 10px;
    border-radius: 6px;
    background: rgba(20, 20, 30, 0.78);
    color: #fff;
    font-size: 11px;
    font-family: inherit;
  }
  .toast-undo {
    border: none;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.92);
    color: rgba(15, 15, 25, 0.88);
    font-size: 11px;
    font-family: inherit;
    font-weight: 600;
    padding: 2px 8px;
    cursor: pointer;
  }
  .toast-undo:hover { background: #fff; }

  .note-actions {
    flex: 0 0 auto;
    display: flex;
    justify-content: flex-end;
    gap: 5px;
    padding: 6px 8px 8px;
  }
  .note-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 4px 8px;
    border-radius: 5px;
    border: none;
    background: rgba(255, 255, 255, 0.55);
    color: rgba(15, 15, 25, 0.72);
    font-size: 11px;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
  }
  .note-btn:hover { background: rgba(255, 255, 255, 0.88); }
  .note-btn--done { background: rgba(255, 255, 255, 0.72); color: rgba(15, 15, 25, 0.88); }
  .note-btn svg { width: 11px; height: 11px; stroke-width: 2.4; }
</style>
