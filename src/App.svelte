<script lang="ts">
  import NoteView from './lib/noteView.svelte';
  import CompletedView from './lib/completedView.svelte';
  import SettingsView from './lib/settingsView.svelte';

  let hash = $state(window.location.hash);
  window.addEventListener('hashchange', () => (hash = window.location.hash));

  const route = $derived(parse(hash));
  function parse(h: string): { name: string; id?: string } {
    const m = h.match(/#\/(note|completed|settings)\??(.*)/);
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
{:else}
  <div></div>
{/if}
