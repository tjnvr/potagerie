// Screenshots of the dev server (fake backend) at 360, 768 and 1440 px, with
// the viewport really set through the DevTools protocol (a headless window
// cannot be narrower than about 500 px).
//   pnpm dev            (in another terminal)
//   node scripts/capturer.mjs "?example&garden&date=2026-06-20" /tmp/captures
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { lancerChrome, pause } from "./chrome.mjs";

const [request = "?example&garden", sortie = "captures", actions = ""] = process.argv.slice(2);
const base = process.env.URL_DEV ?? "http://localhost:5173/";
mkdirSync(sortie, { recursive: true });

const chrome = await lancerChrome();
try {
  await chrome.envoyer("Page.enable");
  for (const [width, height] of [
    [360, 800],
    [768, 1000],
    [1440, 1000],
  ]) {
    await chrome.envoyer("Emulation.setDeviceMetricsOverride", {
      width: width,
      height: height,
      deviceScaleFactor: 1,
      mobile: width < 500,
    });
    await chrome.envoyer("Page.navigate", { url: base + request });
    await pause(2500);
    // Optional script run in the page before the capture, for example a click.
    if (actions) {
      await chrome.evaluate(actions);
      await pause(800);
    }
    const dimensions = await chrome.evaluate("JSON.stringify([document.documentElement.scrollWidth, document.documentElement.scrollHeight])");
    const [, heightPage] = JSON.parse(dimensions);
    const capture = await chrome.envoyer("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: true,
      clip: { x: 0, y: 0, width: width, height: Math.min(heightPage, 2400), scale: 1 },
    });
    writeFileSync(join(sortie, `capture-${width}.png`), Buffer.from(capture.data, "base64"));
    console.log(`capture-${width}.png (${width}x${heightPage}, scrollWidth ${JSON.parse(dimensions)[0]})`);
  }
} finally {
  await chrome.close();
}
