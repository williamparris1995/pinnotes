<script lang="ts">
  // HTML 托盘菜单。结构/图标/颜色对齐 OD 原型。透明窗 + 圆角卡(不上 OS acrylic,
  // 隔离冻结变量)。挂载后量内容高度把窗口收到刚好,去掉底部空白。
  import { onMount } from 'svelte';
  import { invoke, type Note } from './tauri';
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

  const win = getCurrentWindow();
  let count = $state(0);

  // 单条 SVG 内部路径(viewBox 0 0 24 24,stroke=currentColor),{@html} 注入。
  const P = {
    logo: '<path d="M12 17v5"/><path d="M9 3h6l-1 7 3 3v2H7v-2l3-3-1-7z"/>',
    new: '<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>',
    showall:
      '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    hideall:
      '<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>',
    completed: '<circle cx="12" cy="12" r="9"/><polyline points="8 12 11 15 16 9"/>',
    settings:
      '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.36.13.69.34 1 .6"/>',
    quit: '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/>'
  };

  onMount(() => {
    window.addEventListener('keydown', onKey);
    // 失焦兜底:点外部/切窗口时 webview 失焦 → 关窗(与 Rust 端 Focused(false) 双保险)。
    window.addEventListener('blur', () => win.close());
    invoke<Note[]>('list_active')
      .then((a) => (count = a.length))
      .catch(() => {});
    // 量内容高度,把窗口收到刚好(卡满窗 margin 0,去掉底部空白 + 滚动轴)。
    requestAnimationFrame(() => {
      const el = document.querySelector('.menu');
      if (el) {
        const h = Math.ceil(el.getBoundingClientRect().height);
        win.setSize(new LogicalSize(250, h));
      }
    });
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') win.close();
  }
  async function act(action: string) {
    try {
      await invoke('tray_menu_action', { action });
    } catch {
      /* 窗口可能在 await 期间被关,忽略 */
    }
  }
</script>

<div class="menu" role="menu">
  <div class="head">
    <span class="logo"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.logo}</svg></span>
    <span class="htext">
      <span class="hname">PinNotes</span>
      <span class="hsub">置顶便签提醒 · 已驻留</span>
    </span>
  </div>

  <button role="menuitem" onclick={() => act('new')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.new}</svg></span>
    <span class="label">新建便签</span>
    <span class="kbd">Ctrl+N</span>
  </button>
  <button role="menuitem" onclick={() => act('showAll')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.showall}</svg></span>
    <span class="label">显示全部</span>
    <span class="badge">{count}</span>
  </button>
  <button role="menuitem" onclick={() => act('hideAll')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.hideall}</svg></span>
    <span class="label">隐藏全部</span>
  </button>
  <button role="menuitem" onclick={() => act('completed')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.completed}</svg></span>
    <span class="label">已完成…</span>
  </button>

  <div class="sep"></div>

  <button role="menuitem" onclick={() => act('settings')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.settings}</svg></span>
    <span class="label">设置…</span>
  </button>
  <button role="menuitem" class="danger" onclick={() => act('quit')}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.quit}</svg></span>
    <span class="label">退出</span>
  </button>
</div>

<style>
  :global(html, body) {
    height: 100%;
    margin: 0;
    background: #f4f4f7;
    overflow: hidden;
  }
  .menu {
    margin: 0;
    padding: 6px;
    background: #f4f4f7;
    border-radius: 0;
    font-family: 'Segoe UI', system-ui, sans-serif;
    font-size: 13px;
    line-height: 1.2;
    color: #1a1a20;
    display: flex;
    flex-direction: column;
    gap: 1px;
    box-sizing: border-box;
    overflow: hidden;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px 9px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    margin-bottom: 4px;
  }
  .logo {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(145deg, #5b82c0, #4a6fa5);
    border-radius: 5px;
    color: #fff;
    flex-shrink: 0;
  }
  .logo :global(svg) { width: 13px; height: 13px; }
  .htext { display: flex; flex-direction: column; }
  .hname { font-size: 13px; font-weight: 600; }
  .hsub { font-size: 11px; color: #8a8a96; margin-top: 1px; }

  button {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 8px 10px;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    width: 100%;
  }
  button:hover { background: rgba(0, 0, 0, 0.06); }
  .ico {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #5a5a66;
    flex-shrink: 0;
  }
  .ico :global(svg) { width: 15px; height: 15px; }
  .label { flex: 1; }
  .kbd { font-size: 11px; color: #8a8a96; }
  .badge {
    font-size: 11px;
    color: #fff;
    background: #4a6fa5;
    border-radius: 8px;
    padding: 1px 7px;
  }
  .sep { height: 1px; background: rgba(0, 0, 0, 0.08); margin: 4px 8px; }
  .danger { color: #c0392b; }
  .danger .ico { color: #c0392b; }
</style>
