// Frame rate of « Play la season » in headless Chrome (DevTools protocol), on
// the 10-bed bench of P01 at normal density and x4. The page counts its own
// frames (`?fps`, dev only) and publishes the metrics of P01 in
// `window.__potagerieMeasures`. Criterion: mean >= 55 images/s and the slowest
// 1 % of the frames >= 30 images/s.
//   pnpm dev            (in another terminal)
//   node scripts/measure.mjs [width] [height]
import { lancerChrome, pause } from "./chrome.mjs";

const width = Number(process.argv[2] ?? 1440);
const height = Number(process.argv[3] ?? 900);
const base = process.env.URL_DEV ?? "http://localhost:5173/";
const year = new Date().getFullYear();

const chrome = await lancerChrome();
const rows = [];
try {
  await chrome.envoyer("Page.enable");
  await chrome.envoyer("Emulation.setDeviceMetricsOverride", {
    width: width,
    height: height,
    deviceScaleFactor: 1,
    mobile: width < 500,
  });
  for (const density of [1, 4]) {
    for (const playback of [1, 2]) {
      await chrome.envoyer("Page.navigate", {
        url: `${base}?banc&garden&fps&density=${density}&date=${year}-01-01`,
      });
      await pause(2500);
      if (playback === 2) await chrome.evaluate(`document.querySelectorAll(".speed button")[1].click()`);
      await chrome.evaluate(`document.querySelector("[data-play]").click()`);
      let result = null;
      for (let i = 0; i < 120 && !result; i++) {
        await pause(500);
        result = await chrome.evaluate(`JSON.stringify(window.__potagerieMeasures?.[0] ?? null)`);
        result = JSON.parse(result);
      }
      const row = result
        ? { density, speed: playback, ...result }
        : { density, speed: playback, error: "no result in 60 s" };
      rows.push(row);
      console.log(JSON.stringify(row));
    }
  }
  console.log(`\nviewport ${width}x${height}\n`);
  console.log("density | speed | plants | images | average | 1 % lentes | p95 ms | criterion");
  for (const l of rows) {
    if (l.error) console.log(`${l.density} | x${l.speed} | ${l.error}`);
    else {
      const ok = l.average >= 55 && l.bas1pct >= 30;
      console.log(
        `x${l.density} | x${l.speed} | ${l.plants} | ${l.images} | ${l.average.toFixed(1)} | ${l.bas1pct.toFixed(1)} | ${l.p95ms.toFixed(1)} | ${ok ? "OK" : "NON"}`,
      );
    }
  }
} finally {
  await chrome.close();
}
