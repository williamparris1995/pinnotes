<script lang="ts">
  // Task 1 smoke placeholder. Verifies the transparent / borderless / topmost
  // Tauri window. Task 6 replaces this with the hash-routed shell.
  let count = $state(0);
</script>

<div class="note">
  <h1>PinNotes</h1>
  <p>smoke window</p>
  <button onclick={() => count++}>clicked {count}</button>
</div>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    background: transparent;
    font-family: "Segoe UI", system-ui, sans-serif;
  }

  .note {
    box-sizing: border-box;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    /* semi-transparent sticky-note yellow (one of the plan's palette colors) */
    background: rgba(255, 245, 157, 0.85);
    border-radius: 12px;
    color: #333;
    user-select: none;
  }

  h1 {
    font-size: 18px;
    margin: 0;
  }

  p {
    font-size: 12px;
    margin: 0;
    opacity: 0.7;
  }

  button {
    margin-top: 4px;
    font: inherit;
    padding: 2px 10px;
    border: none;
    border-radius: 6px;
    background: rgba(74, 111, 165, 0.9);
    color: #fff;
    cursor: pointer;
  }
</style>
