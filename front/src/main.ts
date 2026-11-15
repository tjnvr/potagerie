import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { appli } from "./lib/etat.svelte";
import { reglerDensiteDeBanc } from "./lib/diorama/plantes";
import { allerA } from "./lib/temps";

async function demarrer() {
  // In a plain browser (`pnpm dev` without Tauri), answer the commands with an
  // in-memory fake, to work on the interface. Never in a build. `?potager`
  // and `?reglages` open that screen directly, for screenshots; `?banc` is
  // the 10-planche bench of the frame rate.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { simulerTauri } = await import("./lib/simulation");
    simulerTauri();
    const params = new URLSearchParams(location.search);
    if (params.has("potager")) appli.onglet = "potager";
    // `?date=2027-06-20`: the fake pretends it is that day (see simulation.ts).
    const date = params.get("date");
    if (date) {
      allerA(date);
      appli.annee = Number(date.slice(0, 4));
    }
    // `?densite=4` draws four times as many plants, for the frame rate bench.
    const densite = Number(params.get("densite"));
    if (densite > 1) reglerDensiteDeBanc(densite);
    if (params.has("reglages")) appli.reglagesOuverts = true;
  }
  mount(App, { target: document.getElementById("app")! });
}

demarrer();
