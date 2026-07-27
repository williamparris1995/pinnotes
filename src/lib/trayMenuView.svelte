<script lang="ts">
  // HTML 托盘菜单。结构/图标/颜色对齐 OD 原型。不透明 + 方角(Win10 上圆角必带毛玻璃边)。
  // 表头显示当前版本号;有新版本时顶部加"新版本"项;底部"检查更新"可手动查(见 ADR-0002)。
  import { onMount } from 'svelte';
  import { invoke, listen, type Note } from './tauri';
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

  const win = getCurrentWindow();
  let count = $state(0);
  let version = $state('');
  let updateVersion = $state<string | null>(null);
  let updating = $state(false);
  let progress = $state<{ d: number; t: number | null } | null>(null);
  // 'idle' | 'checking' | 'latest' —— 手动"检查更新"的反馈态。
  let checkState = $state<'idle' | 'checking' | 'latest'>('idle');

  // 品牌图标(OD 设计的便签+图钉,源在 src-tauri/icons/icon.svg),整张 {@html} 注入。
  const brandIcon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256"><defs><filter id="noteShadow" x="-10%" y="-10%" width="120%" height="135%"><feGaussianBlur in="SourceAlpha" stdDeviation="4"/><feOffset dx="0" dy="6"/><feComponentTransfer><feFuncA type="linear" slope="0.22"/></feComponentTransfer><feMerge><feMergeNode/><feMergeNode in="SourceGraphic"/></feMerge></filter><radialGradient id="pinGrad" cx="35%" cy="32%" r="72%"><stop offset="0%" stop-color="#7fa3d8"/><stop offset="55%" stop-color="#4a6fa5"/><stop offset="100%" stop-color="#3a5a8a"/></radialGradient><linearGradient id="foldGrad" x1="100%" y1="100%" x2="0%" y2="0%"><stop offset="0%" stop-color="#c9a838"/><stop offset="100%" stop-color="#e0bc4a"/></linearGradient></defs><g filter="url(#noteShadow)"><rect x="32" y="104" width="192" height="124" rx="8" fill="#ffe678"/></g><path d="M 200 228 L 216 228 Q 224 228 224 220 L 224 204 Z" fill="url(#foldGrad)"/><g stroke="#c9a838" stroke-width="4" stroke-linecap="round" opacity="0.4"><line x1="56" y1="148" x2="200" y2="148"/><line x1="56" y1="176" x2="176" y2="176"/><line x1="56" y1="204" x2="156" y2="204"/></g><ellipse cx="128" cy="110" rx="24" ry="4" fill="#000" opacity="0.28"/><polygon points="122,88 134,88 130,108 126,108" fill="#2c4470"/><circle cx="128" cy="66" r="26" fill="url(#pinGrad)"/><ellipse cx="119" cy="56" rx="10" ry="6" fill="#ffffff" opacity="0.7"/></svg>`;

  // 单条 SVG 内部路径(viewBox 0 0 24 24,stroke=currentColor),{@html} 注入。
  const P = {
    update: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>',
    new: '<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>',
    showall:
      '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    hideall:
      '<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>',
    completed: '<circle cx="12" cy="12" r="9"/><polyline points="8 12 11 15 16 9"/>',
    settings:
      '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.36.13.69.34 1 .6"/>',
    refresh:
      '<polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>',
    quit: '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/>'
  };

  const checkLabel = $derived(
    checkState === 'checking' ? '检查中…' : checkState === 'latest' ? '已是最新 ✓' : '检查更新'
  );
  const updateLabel = $derived(
    updating
      ? `更新中…${progress && progress.t ? ` ${Math.round((progress.d / progress.t) * 100)}%` : ''}`
      : `新版本 v${updateVersion},点击更新`
  );

  onMount(() => {
    window.addEventListener('keydown', onKey);
    // 失焦兜底:点外部/切窗口时 webview 失焦 → 关窗(与 Rust 端 Focused(false) 双保险)。
    window.addEventListener('blur', () => win.close());
    invoke<string>('get_version')
      .then((v) => (version = v))
      .catch(() => {});
    invoke<Note[]>('list_active')
      .then((a) => (count = a.length))
      .catch(() => {});
    invoke<string | null>('get_update_status')
      .then((v) => (updateVersion = v))
      .catch(() => {});
    listen<{ downloaded: number; total: number | null }>('update-progress', (e) => {
      progress = { d: e.payload.downloaded, t: e.payload.total };
    }).catch(() => {});
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
  async function applyUpdate() {
    if (updating) return;
    updating = true;
    try {
      await invoke('apply_update');
      // macOS/Linux:request_restart 重启;Windows:进程已被 installer 接管退出。
    } catch {
      updating = false; // 失败:放开,允许重试
    }
  }
  async function checkForUpdates() {
    if (checkState === 'checking') return;
    checkState = 'checking';
    try {
      const v = await invoke<string | null>('check_for_updates');
      if (v) {
        updateVersion = v; // 顶部"新版本"项出现
        checkState = 'idle';
      } else {
        checkState = 'latest';
      }
    } catch {
      checkState = 'idle';
    }
  }
</script>

<div class="menu" role="menu">
  <div class="head">
    <span class="logo" aria-label="PinNotes">{@html brandIcon}</span>
    <span class="htext">
      <span class="hrow">
        <span class="hname">PinNotes</span>
        {#if version}<span class="hver">v{version}</span>{/if}
      </span>
      <span class="hsub">让重要的事,一直留在眼前</span>
    </span>
  </div>

  {#if updateVersion}
    <button role="menuitem" class="update" disabled={updating} onclick={applyUpdate}>
      <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.update}</svg></span>
      <span class="label">{updateLabel}</span>
    </button>
    <div class="sep"></div>
  {/if}

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
  <button role="menuitem" class="check" class:ok={checkState === 'latest'} disabled={checkState === 'checking'} onclick={checkForUpdates}>
    <span class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{@html P.refresh}</svg></span>
    <span class="label">{checkLabel}</span>
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
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .logo :global(svg) { width: 24px; height: 24px; }
  .htext { display: flex; flex-direction: column; }
  .hrow { display: flex; align-items: baseline; gap: 6px; }
  .hname { font-size: 13px; font-weight: 600; }
  .hver { font-size: 11px; color: #8a8a96; font-weight: 500; }
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
  button:hover:not(:disabled) { background: rgba(0, 0, 0, 0.06); }
  button:disabled { cursor: default; opacity: 0.6; }
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
  .update { color: #4a6fa5; font-weight: 600; }
  .update .ico { color: #4a6fa5; }
  .check.ok { color: #4caf90; }
  .check.ok .ico { color: #4caf90; }
</style>
