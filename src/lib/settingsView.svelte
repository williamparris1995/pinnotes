<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from './tauri';
  const opts = [1, 2, 5, 10, 30];
  let snooze = $state(2);
  let auto = $state(true);
  onMount(async () => {
    const s = await invoke<Record<string, string>>('get_settings');
    snooze = Number(s.default_snooze_minutes ?? 2);
    auto = await invoke<boolean>('get_autostart');
  });
  function setSnooze(m: number) { snooze = m; invoke('set_settings', { key: 'default_snooze_minutes', value: String(m) }); }
  function setAuto(v: boolean) { auto = v; invoke('set_autostart', { enabled: v }); }
</script>

<main>
  <h2>设置</h2>
  <label>默认隐藏时长</label>
  <div>
    {#each opts as m}
      <button class:active={snooze === m} onclick={() => setSnooze(m)}>{m} 分钟</button>
    {/each}
  </div>
  <label><input type="checkbox" checked={auto} onchange={(e) => setAuto(e.currentTarget.checked)} /> 开机自启</label>
</main>

<style>
  main { font-family: var(--font); padding: 16px; }
  .active { background: var(--accent); color: #fff; }
</style>
