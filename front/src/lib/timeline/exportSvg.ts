// The timeline as a standalone SVG string, for `export_timeline` (S05 §4, §7):
// Rust converts it to PNG (resvg) or PDF (svg2pdf) with the fonts embedded in
// the app (crates/export/fonts), so nothing may depend on the page: no CSS, no
// custom property, no external resource.
//
// `TimelineSvg.svelte` draws everything with attributes. This function takes the
// drawing, drops what only matters on screen (the cursor, handlers, classes,
// focus order) and writes it as text. The export screen (next session) mounts
// `TimelineSvg` for the range the user chose and calls it.

const ATTRIBUTES_D_SCREEN = ["class", "tabindex", "role"];

/**
 * Serialises a drawn timeline.
 * @param svg the `<svg data-timeline>` element rendered by `TimelineSvg.svelte`
 * @param options `cursor`: keep the cursor of the time in the image (off by default)
 */
export function serializeTimeline(svg: SVGSVGElement, options: { cursor?: boolean } = {}): string {
  const copy = svg.cloneNode(true) as SVGSVGElement;
  if (!options.cursor) {
    copy.querySelectorAll("[data-cursor]").forEach((e) => e.remove());
  }
  const all = [copy, ...Array.from(copy.querySelectorAll("*"))];
  for (const e of all) {
    for (const name of ATTRIBUTES_D_SCREEN) e.removeAttribute(name);
    for (const attribute of Array.from(e.attributes)) {
      if (attribute.name.startsWith("data-")) e.removeAttribute(attribute.name);
    }
    e.removeAttribute("style");
  }
  copy.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  const text = new XMLSerializer().serializeToString(copy);
  return `<?xml version="1.0" encoding="UTF-8"?>\n${text}`;
}
