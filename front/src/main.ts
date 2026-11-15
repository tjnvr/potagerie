import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { app } from "./lib/state.svelte";
import { setDensityBench } from "./lib/diorama/plants";
import { allerA } from "./lib/temps";

async function start() {
  // In a plain browser (`pnpm dev` without Tauri), answer the commands with an
  // in-memory fake, to work on the interface. Never in a build. `?garden`
  // and `?settings` open that screen directly, for screenshots; `?banc` is
  // the 10-bed bench of the frame rate.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { simulerTauri } = await import("./lib/simulation");
    simulerTauri();
    const params = new URLSearchParams(location.search);
    if (params.has("garden")) app.tab = "garden";
    // `?date=2027-06-20`: the fake pretends it is that day (see simulation.ts).
    const date = params.get("date");
    if (date) {
      allerA(date);
      app.year = Number(date.slice(0, 4));
    }
    // `?density=4` draws four times as many plants, for the frame rate bench.
    const density = Number(params.get("density"));
    if (density > 1) setDensityBench(density);
    if (params.has("settings")) app.settingsOpen = true;
  }
  mount(App, { target: document.getElementById("app")! });
}

start();
