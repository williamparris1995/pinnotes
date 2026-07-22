// Frontend entry. Task 1 ships a minimal smoke placeholder; Task 6 replaces
// App.svelte with the hash-routed shell and imports the theme here.
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
