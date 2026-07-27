<script lang="ts">
  import { onMount } from 'svelte';
  import NoteView from './lib/noteView.svelte';
  import CompletedView from './lib/completedView.svelte';
  import SettingsView from './lib/settingsView.svelte';
  import TrayMenuView from './lib/trayMenuView.svelte';
  import { initLocale } from './lib/i18n.svelte';

  let hash = $state(window.location.hash);
  window.addEventListener('hashchange', () => (hash = window.location.hash));
  onMount(() => {
    initLocale();
  });

  const route = $derived(parse(hash));
  function parse(h: string): { name: string; id?: string } {
    const m = h.match(/#\/(note|completed|settings|traymenu)\??(.*)/);
    if (!m) return { name: 'blank' };
    const name = m[1];
    const id = new URLSearchParams(m[2]).get('id') ?? undefined;
    return { name, id };
  }
</script>

{#if route.name === 'note' && route.id}
  <NoteView id={route.id} />
{:else if route.name === 'completed'}
  <CompletedView />
{:else if route.name === 'settings'}
  <SettingsView />
{:else if route.name === 'traymenu'}
  <TrayMenuView />
{:else}
  <div></div>
{/if}
