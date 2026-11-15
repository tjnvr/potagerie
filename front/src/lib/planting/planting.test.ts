import { describe, expect, it } from "vitest";
import type { CropView, PlantingPreview, PlantingView, SuggestionView } from "../bindings";
import {
  applySuggestion,
  toggleTransplanting,
  endInput,
  formSincePlanting,
  formEmpty,
  sameZone,
  modeInitial,
  firstShortcutFree,
  segmentsMiniTimeline,
  toPlantingInput,
  zoneAfterClick,
} from "./planting";

const tomato = { id: "tomato", indoor_days: 56 } as CropView;
const carrot = { id: "carrot", indoor_days: 0 } as CropView;

describe("strip of segments", () => {
  it("extends a single segment to the one clicked, either side", () => {
    expect(zoneAfterClick({ start: 3, end: 3 }, 5)).toEqual({ start: 3, end: 5 });
    expect(zoneAfterClick({ start: 3, end: 3 }, 1)).toEqual({ start: 1, end: 3 });
  });

  it("starts again from a click when the zone has several segments or is clicked again", () => {
    expect(zoneAfterClick({ start: 2, end: 4 }, 6)).toEqual({ start: 6, end: 6 });
    expect(zoneAfterClick({ start: 3, end: 3 }, 3)).toEqual({ start: 3, end: 3 });
  });

  it("compares zones", () => {
    expect(sameZone({ start: 1, end: 3 }, { start: 1, end: 3 })).toBe(true);
    expect(sameZone({ start: 1, end: 3 }, null)).toBe(false);
  });

  it("takes the first free shortcut, in order", () => {
    const z = { segment_start: 1, segment_end: 3 };
    expect(firstShortcutFree({ all: null, half: z, third: null, quarter: null })).toEqual({ start: 1, end: 3 });
    expect(firstShortcutFree({ all: null, half: null, third: null, quarter: null })).toBeNull();
  });
});

describe("form to command input", () => {
  it("starts in pots when the catalogue has a time in pots", () => {
    expect(modeInitial(tomato)).toBe("indoor");
    expect(modeInitial(carrot)).toBe("open_ground");
    const f = formEmpty(tomato, "p1", 6, "2027-03-10");
    expect(f.zone).toEqual({ start: 1, end: 6 });
    expect(toPlantingInput(f)).toMatchObject({
      crop_id: "tomato",
      bed_id: "p1",
      segment_start: 1,
      segment_end: 6,
      mode: "indoor",
      date: "2027-03-10",
      manual_end: null,
      date_is_transplanting: false,
    });
  });

  it("sends the days in pots only in the mode indoor", () => {
    const f = { ...formEmpty(carrot, "p1", 6, "2027-03-01"), mode: "indoor" as const, indoorDays: "21" };
    expect(toPlantingInput(f).indoor_days).toBe(21);
    expect(toPlantingInput({ ...f, mode: "plants" }).indoor_days).toBeNull();
    expect(toPlantingInput({ ...f, indoorDays: "" }).indoor_days).toBeNull();
    expect(toPlantingInput({ ...f, transplanting: true }).date_is_transplanting).toBe(true);
    expect(toPlantingInput({ ...f, mode: "plants", transplanting: true }).date_is_transplanting).toBe(false);
  });

  it("fills the form from a suggestion, with its end as an end modified by hand", () => {
    const s: SuggestionView = {
      crop_id: "lettuce",
      mode: "plants",
      date: "2027-08-02",
      planting_out: "2027-08-02",
      harvest_start: "2027-09-21",
      end: "2027-10-06",
      manual_end: "2027-10-06",
      score_tenths: 10,
    };
    const f = applySuggestion(formEmpty(tomato, "p1", 6, "2027-01-01"), s, { start: 1, end: 6 });
    expect(f).toMatchObject({ cropId: "lettuce", mode: "plants", date: "2027-08-02", manualEnd: "2027-10-06" });
  });

  it("reads a planting back for editing", () => {
    const planting = {
      crop_id: "tomato",
      bed_id: "p1",
      segment_start: 2,
      segment_end: 4,
      mode: "indoor",
      date: "2027-03-10",
      end: "2027-09-27",
      manual_end: false,
      pot_days: 56,
    } as PlantingView;
    expect(formSincePlanting(planting)).toMatchObject({ zone: { start: 2, end: 4 }, manualEnd: null, indoorDays: "56" });
    expect(formSincePlanting({ ...planting, manual_end: true, end: "2027-09-01" }).manualEnd).toBe("2027-09-01");
  });

  it("swaps the sowing and transplanting dates with what Rust calculated", () => {
    const preview = { dates: { date: "2027-03-01", planting_out: "2027-03-22" } } as PlantingPreview;
    const f = { ...formEmpty(tomato, "p1", 6, "2027-03-01"), mode: "indoor" as const };
    const r = toggleTransplanting(f, preview);
    expect(r).toMatchObject({ transplanting: true, date: "2027-03-22" });
    expect(toggleTransplanting(r, preview)).toMatchObject({ transplanting: false, date: "2027-03-01" });
    // Without a preview the date stays.
    expect(toggleTransplanting(f, null)).toMatchObject({ transplanting: true, date: "2027-03-01" });
  });

  it("does not call the end « by hand » when it is the calculated one", () => {
    expect(endInput("2027-10-11", "2027-10-11")).toBeNull();
    expect(endInput("2027-10-06", "2027-10-11")).toBe("2027-10-06");
    expect(endInput("", "2027-10-11")).toBeNull();
  });

  it("cuts the mini-timeline from the dates of Rust", () => {
    const dates = {
      sowing: "2027-03-10",
      planting_out: "2027-05-05",
      harvest_start: "2027-07-19",
      end: "2027-09-27",
    } as NonNullable<PlantingPreview["dates"]>;
    expect(segmentsMiniTimeline(dates, "indoor").map((s) => s.phase)).toEqual(["pots", "growth", "harvest"]);
    expect(segmentsMiniTimeline(dates, "indoor").map((s) => s.days)).toEqual([56, 75, 70]);
    expect(segmentsMiniTimeline(dates, "plants").map((s) => s.phase)).toEqual(["growth", "harvest"]);
  });
});
