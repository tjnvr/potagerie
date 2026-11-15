# POT-P01 · Isometric rendering prototype — v0.2

> Version: 0.2 (draft) · Date: 2026-11-15 · Author: Tim
> Online page: https://claude.ai/artifact/PBrzt3qta6sHKisnhfZWMv
> v0.2 is the English translation of v0.1 (page, texts and README). The question, the protocol and the results do not change; the last paragraph of the decision now refers to the Tauri + Svelte stack (S05 v0.12).

## Question

Does the diorama (10 beds, the 6 crops of S02) stay smooth during "Play the season", on a computer and on a phone? Which engine to keep: **SVG** or **Canvas**?

## Content

| File | Role |
|---|---|
| `index.html` | Standalone page: diorama, season playback in 15 s, choice of the engine and of the density, frames/s measure |

Open `index.html` in a browser, or the online page.

## Protocol

For each device (computer, phone): play a full season for each engine × density combination (Normal, × 2, × 4), then "Copy the results".

Criterion: **average ≥ 55 frames/s** and **slowest 1 % of frames ≥ 30 frames/s** at Normal density.

## Results

| Device | Rendering | Density | Plants | Average | Slowest 1 % | p95 | Verdict |
|---|---|---|---|---|---|---|---|
| Mac (screen 1512×982, 120 Hz) | SVG | Normal | 132 | 120 fps | 106 fps | 9.3 ms | Smooth |
| Mac (screen 1512×982, 120 Hz) | Canvas | Normal | 132 | 120 fps | 106 fps | 9.2 ms | Smooth |
| Phone | SVG | Normal | | | | | to measure |
| Phone | SVG | × 4 | | | | | to measure |

On the Mac, both engines cap at the refresh rate of the screen: at 132 plants, they cannot be told apart.

## Decision

**SVG kept, provisionally.** With equal performance on a computer, it simplifies the editor (tapping, dragging a bed), zooming and the vector export.

To confirm on a phone (SVG Normal and × 4). Fallback if the SVG stutters: Canvas for the animated plants, a transparent SVG layer on top for the beds that are tapped.

With Tauri + Svelte (S05), the interface renders the SVG of the diorama in the webview; the season animation also stays in the webview, driven by `requestAnimationFrame`. No pixel image is generated on the Rust side, except for the timeline export (resvg).
