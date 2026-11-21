// @vitest-environment happy-dom
import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import type { BedView, PlantingView } from "../bindings";
import { dayOf } from "../dates";
import { serializeTimeline } from "./exportSvg";
import TimelineSvg from "./TimelineSvg.svelte";
import { barOf, slotOf, type Row } from "./geometry";

const bed: BedView = {
  id: "p1",
  name: "The tomatoes",
  bed_type: "bed",
  x: 0,
  y: 0,
  length_cm: 300,
  width_cm: 120,
  segments: 6,
  plantings_deleted: [],
  name_shared: false,
};

const tomato = {
  id: "s1",
  crop_id: "tomato",
  bed_id: "p1",
  segment_start: 1,
  segment_end: 3,
  mode: "indoor",
  date: "2027-03-10",
  sowing: "2027-03-10",
  planting_out: "2027-05-05",
  harvest_start: "2027-07-19",
  end: "2027-09-27",
  harvests: [{ start: "2027-07-19", end: "2027-09-27" }],
  manual_end: false,
  pot_days: 56,
  rows: 1,
  plants_by_row: 2,
  appearance_today: { appearance: "nothing" },
} as PlantingView;

const rows: Row[] = [
  {
    bed,
    bars: [{ ...barOf(tomato), lane: 0 }],
    slots: [
      {
        ...slotOf({
          bed_id: "p1",
          segment_start: 1,
          segment_end: 6,
          start: "2027-09-27",
          end: "2027-11-15",
          suggestions: [],
        }),
        lane: 0,
      },
    ],
    absences: [],
    lanes: 1,
  },
];

let instance: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (instance) unmount(instance);
  instance = null;
  document.body.innerHTML = "";
});

function draw(): SVGSVGElement {
  instance = mount(TimelineSvg, {
    target: document.body,
    props: {
      rows,
      scale: { startDay: dayOf("2027-01-01"), pxByDay: 2.5, width: 912 },
      today: "2027-06-20",
      cursor: "2027-06-20",
      interactive: true,
      yearShown: 2027,
    },
  });
  return document.querySelector("svg[data-timeline]") as SVGSVGElement;
}

describe("timeline export", () => {
  it("is a standalone SVG: no CSS, no style, no custom property, no external resource", () => {
    const text = serializeTimeline(draw());
    expect(text.startsWith('<?xml version="1.0" encoding="UTF-8"?>')).toBe(true);
    expect(text).toContain('xmlns="http://www.w3.org/2000/svg"');
    expect(text).not.toMatch(/<style/i);
    expect(text).not.toMatch(/\sstyle=/);
    expect(text).not.toMatch(/\sclass=/);
    expect(text).not.toContain("var(--");
    expect(text).not.toMatch(/https?:\/\/(?!www\.w3\.org)/);
    expect(text).not.toContain("<link");
    expect(text).not.toContain("@import");
    expect(text).not.toMatch(/href=/);
  });

  it("names the embedded fonts and draws the three phases with distinct patterns", () => {
    const text = serializeTimeline(draw());
    expect(text).toContain('font-family="Figtree');
    expect(text).toContain("Fredoka");
    expect(text).toContain('id="timeline-pots-tomato"');
    expect(text).toContain('id="timeline-growth-tomato"');
    expect(text).toContain('fill="url(#timeline-pots-tomato)"');
    expect(text).toContain('fill="url(#timeline-growth-tomato)"');
    expect(text).toContain('fill="#E5533D"');
    expect(text).toContain("The tomatoes");
  });

  it("drops the cursor and the screen-only attributes, unless asked", () => {
    const svg = draw();
    expect(serializeTimeline(svg)).not.toContain("data-");
    expect(svg.querySelector("[data-cursor]")).not.toBeNull();
    const without = serializeTimeline(svg);
    const withMarker = serializeTimeline(svg, { cursor: true });
    expect(withMarker.length).toBeGreaterThan(without.length);
    // The live drawing is left untouched.
    expect(svg.querySelector("[data-bar]")).not.toBeNull();
  });

  it("parses back as XML", () => {
    const text = serializeTimeline(draw());
    const doc = new DOMParser().parseFromString(text, "image/svg+xml");
    expect(doc.documentElement.nodeName).toBe("svg");
    expect(doc.querySelectorAll("parsererror")).toHaveLength(0);
  });
});
