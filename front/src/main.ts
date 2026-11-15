import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { appli } from "./lib/etat.svelte";

async function demarrer() {
  // In a plain browser (`pnpm dev` without Tauri), answer the commands with an
  // in-memory fake, to work on the interface. Never in a build. `?potager`
  // and `?reglages` open that screen directly, for screenshots.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { simulerTauri } = await import("./lib/simulation");
    simulerTauri();
    const params = new URLSearchParams(location.search);
    if (params.has("potager")) appli.onglet = "potager";
    if (params.has("reglages")) appli.reglagesOuverts = true;
  }
  mount(App, { target: document.getElementById("app")! });
}

demarrer();
