<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from './tauri';

  // Prototype pinnotes-8c76 · Section 03 SettingsWindow: a snooze segmented
  // control (1/2/5/10/30 分钟) and an 开机自启 toggle. Dynamic behavior is
  // unchanged: load on mount, persist on change.
  const opts = [1, 2, 5, 10, 30];
  let snooze = $state(2);
  let auto = $state(true);

  onMount(async () => {
    const s = await invoke<Record<string, string>>('get_settings');
    snooze = Number(s.default_snooze_minutes ?? 2);
    auto = await invoke<boolean>('get_autostart');
  });

  function setSnooze(m: number) {
    snooze = m;
    invoke('set_settings', { key: 'default_snooze_minutes', value: String(m) });
  }
  function setAuto(v: boolean) {
    auto = v;
    invoke('set_autostart', { enabled: v });
  }
  // The prototype's toggle is a styled <div>; expose it as a real switch so
  // keyboard + screen-reader users can operate it (and to satisfy Svelte's
  // a11y checks without suppressing them).
  function onToggleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      setAuto(!auto);
    }
  }
</script>

<main>
  <div class="set-row">
    <div>
      <div class="set-label">默认隐藏时长</div>
      <div class="set-hint">点「隐藏」后便签暂时收起的时间 · 到点未完成自动弹回原位</div>
    </div>
    <div class="set-controls">
      <div class="seg">
        {#each opts as m}
          <button type="button" class="seg-opt" class:active={snooze === m} onclick={() => setSnooze(m)}>{m}</button>
        {/each}
      </div>
      <span class="seg-suffix">分钟</span>
    </div>
  </div>
  <div class="set-row">
    <div>
      <div class="set-label">开机自启</div>
      <div class="set-hint">Windows 登录后自动启动并驻留系统托盘</div>
    </div>
    <div
      class="toggle"
      class:off={!auto}
      role="switch"
      aria-checked={auto}
      aria-label="开机自启"
      tabindex="0"
      onclick={() => setAuto(!auto)}
      onkeydown={onToggleKey}
    ></div>
  </div>
</main>

<style>
  /* Ported from OD prototype (pinnotes-8c76) — exact values for the settings
     window body. The auxiliary windows use native OS chrome (titlebar /
     minimize / close come from the OS), so only the field rows are rendered. */
  main {
    --canvas-soft: #fafafb;
    --ink: #1a1a20;
    --ink-2: #5a5a66;
    --ink-3: #8a8a96;
    --line-2: #ededf2;
    font-family: var(--font);
    background: #fcfcfe;
    min-height: 100%;
    box-sizing: border-box;
    padding: 4px 0;
  }

  .set-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 18px;
    border-bottom: 1px solid var(--line-2);
    gap: 24px;
  }
  .set-row:last-child { border-bottom: none; }
  .set-label { font-size: 13px; font-weight: 500; color: var(--ink); }
  .set-hint { font-size: 11.5px; color: var(--ink-3); margin-top: 3px; line-height: 1.5; }
  .set-controls { display: flex; align-items: center; }

  .seg {
    display: inline-flex;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 7px;
    padding: 2px;
    gap: 1px;
  }
  .seg-opt {
    padding: 5px 12px;
    font-size: 12px;
    font-family: inherit;
    font-weight: 500;
    color: var(--ink-2);
    background: transparent;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    transition: all 0.12s;
    font-variant-numeric: tabular-nums;
    min-width: 32px;
  }
  .seg-opt:hover:not(.active) { color: var(--ink); }
  .seg-opt.active {
    background: #fff;
    color: var(--accent);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.14), 0 0 0 1px rgba(0, 0, 0, 0.03);
    font-weight: 600;
  }
  .seg-suffix { font-size: 11px; color: var(--ink-3); margin-left: 8px; }

  .toggle {
    position: relative;
    width: 40px;
    height: 21px;
    background: var(--accent);
    border-radius: 12px;
    cursor: pointer;
    transition: background 0.18s;
    flex-shrink: 0;
  }
  .toggle::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 22px;
    width: 15px;
    height: 15px;
    background: #fff;
    border-radius: 50%;
    transition: left 0.18s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }
  .toggle.off { background: rgba(0, 0, 0, 0.28); }
  .toggle.off::after { left: 3px; }
</style>
