// The timeline as a standalone SVG string, for `exporter_frise` (S05 §4, §7):
// Rust converts it to PNG (resvg) or PDF (svg2pdf) with the fonts embedded in
// the app (crates/export/fonts), so nothing may depend on the page: no CSS, no
// custom property, no external resource.
//
// `FriseSvg.svelte` draws everything with attributes. This function takes the
// drawing, drops what only matters on screen (the cursor, handlers, classes,
// focus order) and writes it as text. The export screen (next session) mounts
// `FriseSvg` for the range the user chose and calls it.

const ATTRIBUTS_D_ECRAN = ["class", "tabindex", "role"];

/**
 * Serialises a drawn timeline.
 * @param svg the `<svg data-frise>` element rendered by `FriseSvg.svelte`
 * @param options `curseur`: keep the cursor of the time in the image (off by default)
 */
export function serialiserFrise(svg: SVGSVGElement, options: { curseur?: boolean } = {}): string {
  const copie = svg.cloneNode(true) as SVGSVGElement;
  if (!options.curseur) {
    copie.querySelectorAll("[data-curseur]").forEach((e) => e.remove());
  }
  const tous = [copie, ...Array.from(copie.querySelectorAll("*"))];
  for (const e of tous) {
    for (const nom of ATTRIBUTS_D_ECRAN) e.removeAttribute(nom);
    for (const attribut of Array.from(e.attributes)) {
      if (attribut.name.startsWith("data-")) e.removeAttribute(attribut.name);
    }
    e.removeAttribute("style");
  }
  copie.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  const texte = new XMLSerializer().serializeToString(copie);
  return `<?xml version="1.0" encoding="UTF-8"?>\n${texte}`;
}
