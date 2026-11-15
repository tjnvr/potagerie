// Frame rate of « Jouer la saison » in headless Chrome (DevTools protocol), on
// the 10-planche bench of P01 at normal density and x4. The page counts its own
// frames (`?fps`, dev only) and publishes the metrics of P01 in
// `window.__potagerieMesures`. Criterion: mean >= 55 images/s and the slowest
// 1 % of the frames >= 30 images/s.
//   pnpm dev            (in another terminal)
//   node scripts/mesurer.mjs [largeur] [hauteur]
import { lancerChrome, pause } from "./chrome.mjs";

const largeur = Number(process.argv[2] ?? 1440);
const hauteur = Number(process.argv[3] ?? 900);
const base = process.env.URL_DEV ?? "http://localhost:5173/";
const annee = new Date().getFullYear();

const chrome = await lancerChrome();
const lignes = [];
try {
  await chrome.envoyer("Page.enable");
  await chrome.envoyer("Emulation.setDeviceMetricsOverride", {
    width: largeur,
    height: hauteur,
    deviceScaleFactor: 1,
    mobile: largeur < 500,
  });
  for (const densite of [1, 4]) {
    for (const lecture of [1, 2]) {
      await chrome.envoyer("Page.navigate", {
        url: `${base}?banc&potager&fps&densite=${densite}&date=${annee}-01-01`,
      });
      await pause(2500);
      if (lecture === 2) await chrome.evaluer(`document.querySelectorAll(".vitesse button")[1].click()`);
      await chrome.evaluer(`document.querySelector("[data-jouer]").click()`);
      let resultat = null;
      for (let i = 0; i < 120 && !resultat; i++) {
        await pause(500);
        resultat = await chrome.evaluer(`JSON.stringify(window.__potagerieMesures?.[0] ?? null)`);
        resultat = JSON.parse(resultat);
      }
      const ligne = resultat
        ? { densite, vitesse: lecture, ...resultat }
        : { densite, vitesse: lecture, erreur: "no result in 60 s" };
      lignes.push(ligne);
      console.log(JSON.stringify(ligne));
    }
  }
  console.log(`\nviewport ${largeur}x${hauteur}\n`);
  console.log("densite | vitesse | plantes | images | moyenne | 1 % lentes | p95 ms | critere");
  for (const l of lignes) {
    if (l.erreur) console.log(`${l.densite} | x${l.vitesse} | ${l.erreur}`);
    else {
      const ok = l.moyenne >= 55 && l.bas1pct >= 30;
      console.log(
        `x${l.densite} | x${l.vitesse} | ${l.plantes} | ${l.images} | ${l.moyenne.toFixed(1)} | ${l.bas1pct.toFixed(1)} | ${l.p95ms.toFixed(1)} | ${ok ? "OK" : "NON"}`,
      );
    }
  }
} finally {
  await chrome.fermer();
}
