<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';
  import { t } from './i18n.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { renderMd } from './markdown';

  let { id }: { id: string } = $props();
  let note = $state<Note | null>(null);
  let draft = $state('');
  let loadError = $state(false);
  // #6: pending completion shows an undo toast instead of closing immediately.
  let pendingComplete = $state(false);
  let completeTimer: ReturnType<typeof setTimeout> | null = null;

  // #3/#4: color presets (match the .note-yellow/pink/blue/green CSS). Labels
  // are localized via t(`note.color.${id}`).
  const COLORS = ['yellow', 'pink', 'blue', 'green'] as const;
  const SIZE_NORMAL = { w: 240, h: 170 };
  const SIZE_LARGE = { w: 360, h: 260 };
  // #2: per-note hide duration cycled in the toolbar; takes effect on next 隐藏.
  const SNOOZE_OPTS = [1, 2, 5, 10, 30, 60] as const;
  let isLarge = $derived(note ? note.w >= 340 : false);

  let taRef = $state<HTMLTextAreaElement | null>(null);

  // Markdown 编辑/渲染状态机 (design.md ADR-1/2)。editing 仅 markdown 开时有意义:
  // markdown 关 → 始终 textarea(现状);markdown 开 → 非编辑渲染 HTML,点内容进编辑。
  let editing = $state(false);
  let showTextarea = $derived(!note || !note.markdown || editing || note.content === '');
  let renderedHtml = $derived(note?.markdown && note.content ? renderMd(note.content) : '');

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

  // Render the loaded content and focus empty notes. Setting the textarea
  // value explicitly here is the robust path: bind:value alone didn't reflect
  // the late onMount load on some notes (copied/reactivated), leaving them
  // blank despite having content in the DB.
  $effect(() => {
    if (!taRef || !note) return;
    if (taRef.value !== note.content) {
      taRef.value = note.content;
      draft = note.content;
    }
    if (note.content === '') {
      taRef.focus();
      const len = taRef.value.length;
      taRef.selectionStart = len;
      taRef.selectionEnd = len;
    }
  });

  // Save on blur — clicking colour/size/hidden/complete/markdown blurs the
  // textarea and persists any change.
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

  // Markdown per-note 开关 (ADR-2)。关 → 纯 textarea(现状,打开就打字)。
  function toggleMarkdown() {
    if (!note) return;
    const on = !note.markdown;
    invoke('set_markdown', { id, on });
    note.markdown = on;
    if (!on) editing = false;
  }

  // 点渲染态 → 进编辑(显示源码)。textarea 渲染后再 focus (requestAnimationFrame)。
  function enterEdit() {
    editing = true;
    requestAnimationFrame(() => {
      taRef?.focus();
      if (taRef) {
        const len = taRef.value.length;
        taRef.selectionStart = taRef.selectionEnd = len;
      }
    });
  }

  // 失焦:保存 + markdown 开则回渲染态。
  async function handleFocusout() {
    await commit();
    if (note?.markdown) editing = false;
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
  <div class="note note-error">{t('note.loadError')}</div>
{:else if note}
  <article class="note note-{note.color}">
    <div
      class="note-grip"
      role="button"
      tabindex="0"
      aria-label={t('note.drag')}
      onpointerdown={() => getCurrentWindow().startDragging()}
    ></div>
    <div class="note-toolbar">
      <div class="color-dots">
        {#each COLORS as cid (cid)}
          <button
            type="button"
            class="color-dot color-dot-{cid}"
            class:active={note.color === cid}
            aria-label={t('note.colorAria', { label: t(`note.color.${cid}`) })}
            aria-pressed={note.color === cid}
            onclick={() => setColor(cid)}
          ></button>
        {/each}
      </div>
      <div class="toolbar-tools">
        <button
          type="button"
          class="md-btn"
          class:on={note.markdown}
          onclick={toggleMarkdown}
          title={t('note.markdown')}
          aria-label={t('note.markdown')}
          aria-pressed={note.markdown}
        >M↓</button>
        <button type="button" class="size-btn" title={t('note.snoozeTitle')} onclick={cycleSnooze}>
          {t('note.snoozeMinutes', { count: note.snooze_minutes })}
        </button>
        <button type="button" class="size-btn" onclick={toggleSize}>
          {isLarge ? t('note.sizeSmall') : t('note.sizeLarge')}
        </button>
      </div>
    </div>
    <div class="note-body">
      {#if showTextarea}
        <textarea
          bind:value={draft}
          bind:this={taRef}
          oninput={onInput}
          onfocusout={handleFocusout}
          placeholder={t('note.placeholder')}
        ></textarea>
      {:else}
        <div
          class="note-md"
          role="textbox"
          tabindex="0"
          onclick={enterEdit}
          onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && enterEdit()}
        >
          {@html renderedHtml}
          <span class="edit-hint">{t('note.clickToEdit')}</span>
        </div>
      {/if}
    </div>
    {#if pendingComplete}
      <div class="note-toast" role="status">
        <span>{t('note.completedToast')}</span>
        <button type="button" class="toast-undo" onclick={undoComplete}>{t('note.undo')}</button>
      </div>
    {/if}
    <div class="note-actions">
      <button class="note-btn" onclick={() => invoke('hide_note', { id })}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15 14"/></svg>
        {t('note.hide')}
      </button>
      <button class="note-btn note-btn--done" onclick={onComplete}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        {t('note.complete')}
      </button>
    </div>
  </article>
{/if}

<style>
  /* Solid opaque sticky note (no translucency). The card fills its window;
     the window's w/h defines the note size (default 240x170). No border /
     radius / shadow: the window is opaque, so those would either clip or show
     the window background — a plain solid rectangle is what's wanted. */
  :global(html, body, #app) { height: 100%; margin: 0; }

  .note {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
  }
  .note-error { display: block; padding: 16px; background: #eee; color: #b00; }

  .note-yellow { background: var(--c-yellow); }
  .note-pink   { background: var(--c-pink); }
  .note-blue   { background: var(--c-blue); }
  .note-green  { background: var(--c-green); }

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
  .toolbar-tools { display: flex; gap: 4px; align-items: center; }
  .color-dot {
    width: 14px;
    height: 14px;
    padding: 0;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.18);
    cursor: pointer;
  }
  .color-dot-yellow { background: var(--c-yellow); }
  .color-dot-pink   { background: var(--c-pink); }
  .color-dot-blue   { background: var(--c-blue); }
  .color-dot-green  { background: var(--c-green); }
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
  /* Markdown 开关 (ADR-2):默认描边,激活 accent 实心。挤在色点与 snooze 间。 */
  .md-btn {
    width: 20px;
    height: 16px;
    padding: 0;
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.18);
    background: rgba(255, 255, 255, 0.5);
    color: rgba(15, 15, 25, 0.6);
    font-size: 9px;
    font-weight: 800;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: ui-monospace, monospace;
  }
  .md-btn:hover { background: rgba(255, 255, 255, 0.85); }
  .md-btn.on { background: var(--accent); color: #fff; border-color: var(--accent); }

  .note-body {
    flex: 1 1 auto;
    min-height: 0;
  }
  .note-body textarea {
    display: block;
    width: 100%;
    height: 100%;
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

  /* 渲染态 markdown 排版 (design-system v1 token,240×170 小窗适配)。
     {:global} 包裹:{@html} 插入的元素无 scoped hash,须全局(限定 .note-md 内)。 */
  .note-md {
    height: 100%;
    overflow: auto;
    scrollbar-width: none;
    padding: 2px 16px 4px;
    font-family: inherit;
    font-size: 15px;
    font-weight: 600;
    line-height: 1.4;
    color: rgba(15, 15, 25, 0.86);
    cursor: text;
    position: relative;
  }
  .note-md::-webkit-scrollbar { display: none; }
  .note-md :global(h1) { font-size: 16px; font-weight: 800; margin: 2px 0; line-height: 1.25; }
  .note-md :global(h2) { font-size: 15px; font-weight: 800; margin: 2px 0; }
  .note-md :global(h3) { font-size: 14px; font-weight: 700; margin: 1px 0; }
  .note-md :global(ul),
  .note-md :global(ol) { margin: 2px 0; padding-left: 18px; }
  .note-md :global(li) { margin: 1px 0; }
  .note-md :global(blockquote) {
    margin: 2px 0;
    padding-left: 8px;
    border-left: 2px solid rgba(0, 0, 0, 0.16);
    color: rgba(15, 15, 25, 0.6);
  }
  .note-md :global(code) {
    background: rgba(0, 0, 0, 0.07);
    border-radius: 3px;
    padding: 0 4px;
    font-family: ui-monospace, monospace;
    font-size: 13px;
  }
  .note-md :global(del) { color: rgba(15, 15, 25, 0.6); }
  .note-md :global(hr) { border: none; border-top: 1px solid rgba(0, 0, 0, 0.16); margin: 6px 0; }
  .note-md :global(p) { margin: 2px 0; }
  .note-md :global(strong) { font-weight: 800; }
  .edit-hint {
    position: absolute;
    right: 8px;
    bottom: 6px;
    font-size: 10px;
    font-weight: 500;
    background: rgba(20, 20, 30, 0.7);
    color: #fff;
    padding: 2px 6px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.12s;
    pointer-events: none;
  }
  .note-md:hover .edit-hint { opacity: 1; }

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
