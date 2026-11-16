# POT-S03 · Rules and calculations — v0.14

> Version: 0.14 (draft) · Date: 2026-11-16 · Author: Tim
> References: [S01](POT-S01_functional-spec_v1.6.md) · [S02](POT-S02_crop-catalogue_v0.4.md) · [S05](POT-S05_technical-spec_v0.13.md)
> v0.14 settles the inconsistencies found by the audit (section 17, sections 15 and 16). v0.13 sorts the last tie-break of the suggestions (section 9) with the name shown in the language of the interface. v0.12 lets the winter crops (garlic, spring onion) be suggested in a slot that reaches 15 November (section 9, steps 3 and 4); no other rule changes. v0.11 is the English translation of v0.10; no rule and no expected value changes. Field names, error codes and commands follow the code. History: v0.6 specified, while writing `crates/plan`, the cases v0.5 left open. v0.7 added, while writing the editor, the placement rules of the beds and the validation of the plot (section 15). v0.8 added, while writing the settings, the validation of an imported plan (section 13) and the rules of the annotations (section 16). v0.9 added, while writing the timeline export, the "to resolve" list kept after an import (section 13, questions 1 and 7 settled for lot 1). v0.10 keeps the test cases on the reference catalogue.

This spec fixes **every calculation** of the app, precisely enough to be coded in Rust (`crates/plan`) and tested. The cases of section 12 are the first unit tests.

---

## 1. Conventions

- **Plan dates = civil dates**, with no time and no time zone: a sowing "on 10 March" is 10 March everywhere. They are never converted to UTC, otherwise 10 March at 00:00 UTC would become the evening of 9 March west of Greenwich.
- **Technical timestamps in UTC**: the last change of a document or of a piece of data, for example.
- All durations are in days.
- **Today** is a parameter of the calculations, not a direct read of the clock, so that the tests are reproducible. It is the local date of the device.
- `d + n` is the date n days after d.
- A catalogue date `MM-DD` is placed in a year Y. If it falls on `02-29` in a non-leap year, `02-28` is used.
- **Occupation of a bed**: **half-open** interval `[planting out, end[`. A planting may therefore start on the very day the previous one ends.

---

## 2. Dates of a planting

**Inputs**: the start mode, a date `s`, and possibly an end edited by hand.

**Durations**: `indoor`, `after` and `duration` are **copied from the catalogue (S02) into the planting when it is created**, and copied again when the crop changes. A catalogue update therefore never shifts an existing planting.

| Mode | Stored `s` | Planting out `m` | Start of harvest `r` |
|---|---|---|---|
| I sow under cover (`indoor`) | sowing date | `s + A` | `m + after` |
| I buy plants (`plants`) | planting date | `s` | `m + after` |
| I sow in the open ground (`open_ground`) | sowing date | `s` | `m + after + indoor` |

- **"I rather know the transplanting date"** only changes the input: if the user enters a transplanting date `x`, the app stores the sowing date `s = x − A`.
- `A` is the time under cover: `indoor` if it is greater than 0. For a crop the catalogue does not grow in pots (`indoor = 0`, carrot or bean), the indoor mode shows the field "How many days in pots?": the value entered, at least 1 day, is stored with the planting and serves as `A`.
- **Changing the mode** of a planting copies the time under cover `A` again from the catalogue: a carrot switched from "under cover" (21 days entered) to "open ground" adds nothing any more, as in T3.
- **Sowing in the open ground**: the plant spends in the ground the phase it would have spent in a pot. `indoor` is therefore added to the time until harvest, with 0 for the carrot or the bean.
- **End** `f = r + duration`, unless it was edited by hand. An entered end must be after `r`. A planting whose end `f` is not after `m` occupies no day and is refused.
- If `s` or the mode is changed, `m`, `r` and `f` are recomputed, except an end edited by hand (S01, F-PLA-2).
- **Dragging a bar** in the timeline by n days shifts the entered date `s` and the manual end, if any, by n days. If the shifted planting is no longer valid (section 5), the shift is refused and the bar goes back to its place.

The **phases of the timeline** are:
- pots, from sowing to `m`, indoor mode only;
- growth, from `m` to `r`;
- harvest, from `r` to `f`.

Only growth and harvest occupy the bed.

---

## 3. Visual stages (garden view)

At a date `d`:

| Situation | Display |
|---|---|
| `d < m`, indoor mode and `d ≥ sowing` | Label planted on the zone |
| `d < m` otherwise, or `d ≥ f` | Nothing |
| `m ≤ d < f` | Plant, at the stage given by `p = (d − m) / (r − m)` |

| `p` | Stage |
|---|---|
| `< 0.15` | Sowing |
| `0.15 – 0.50` | Young shoot |
| `0.50 – 1` | Adult |
| `≥ 1` | Harvest |

Each range includes its lower bound: `p = 0.15` is "Young shoot", `p = 0.50` is "Adult". If `r = m` (no delay before the harvest), the plant is directly at the "Harvest" stage.

A planting of **bought plants** never goes below the "Young shoot" stage.

---

## 4. Segments and zone

- **Size of a segment** = cell size of the plot (25, 50 or 100 cm). It is fixed when the plot is created and never changes, so that the zones stay valid.
- The **length** of a bed is the side cut into segments. It is a whole number of cells, and the number of segments is `n = length ÷ cell size`. The **width** is free, to the centimetre.
- The segments are numbered 1 to n along the length.
- A zone is an interval `[t1, t2]` with `1 ≤ t1 ≤ t2 ≤ n`.
- **Shortcuts** ½, ⅓ and ¼:
  - the target size is `k = max(1, round(n × fraction))`, rounding up for a half (n = 6, ¼: 1.5 → 2);
  - the chosen zone is the first run of k segments that are free over the whole planned occupation of the planting, and that fits in the bed every year this occupation touches;
  - if there is none, the shortcut is greyed out.
- If a bed is **shortened** to n′ segments, every planting with `t2 > n′` that occupies the changed year or a later one is flagged before validation (S01, F-PLN-6).

---

## 5. Validity of a planting and conflicts

A planting is **valid** if its bed exists in the layout of every year its occupation `[m, f[` touches (a planting that ends on 1 January does not touch the new year), if its zone fits in the bed (`t2 ≤ n`, with the `n` of each of these years), and if it conflicts with no other planting of the bed.

Two plantings a and b of the same bed **conflict** if their zones overlap and their occupations overlap:

```
a.t1 ≤ b.t2  and  b.t1 ≤ a.t2      (zones)
a.m  < b.f   and  b.m  < a.f       (half-open occupations)
```

On a conflict, validation is refused. The app shows the common segments and the common period, `[max(a.m, b.m), min(a.f, b.f)[`.

**Deleting a bed from year Y** (S01, F-PLN-4): all its plantings whose end `f` is after 1 January of Y are deleted, including those that started before Y. Plantings that end on 1 January of Y at the latest are kept. The layouts of this bed saved for a year after Y are erased with it.

---

## 6. Quantities

For a bed of width `W` (in cm), a zone of `k` segments and a cell size `c`:

```
rows           = max(1, ⌊W ÷ row_cm⌋)
plants per row = max(1, ⌊(k × c) ÷ spacing_cm⌋)
quantity       = rows × plants per row
```

Display: "≈ 21 plants, 1 row × 21".

---

## 7. Layouts of the plan

- The layout of a bed is only saved for the **years in which it changes**.
- The layout of a bed in year Y is its last layout saved for a year ≤ Y. If it has none, the bed does not exist yet in Y.
- A deletion from Y on is an "absent" layout saved in Y.
- A change of a bed in year Y replaces its layout of Y. It then holds for the following years **until the next saved layout of that same bed**, which stays as it is. The other beds are not touched.
- The layouts of a past year cannot be changed (Y < year of today); see section 17 for what stays editable.
- A year never holds more than 10 beds (S01, F-PLN-1): adding an 11th, or bringing back an absent bed that would bring a year to 11, is refused.
- The length of a layout is a whole number of cells, at least one.

---

## 8. Free slots

The calculation is done for each bed and for each year Y in which it exists in the layout of Y.

1. **Search window**: `[Y-03-01, Y-11-15]`.
2. **Cutting**: the window is cut at the dates `m` and `f` of all the plantings of the bed. On each sub-period, the set of occupied segments is constant.
3. A **slot** is a triple `(zone, start, end)` that meets four conditions:
   - all the segments of the zone are free every day of `[start, end]`;
   - `end − start + 1 > 35` days;
   - it is **maximal in time**: `start` cannot be moved earlier nor `end` later without occupying a segment of the zone, or leaving the window;
   - it is **maximal in width**: the zone cannot be widened over the whole period.
4. Only the slots **not contained** in another one are kept, in zone as in period.

Display:
- in the timeline, a dotted rectangle on the segments and the period of the slot;
- in the garden view, a "+ Free" sign on the bed if a slot contains the date of the cursor.

---

## 9. Suggestions

For a slot of zone Z, from day D to day F, of length `N = F − D + 1`, each crop of the catalogue is evaluated as follows:

1. **Planting-out date**: `m` is the earliest date of `[D, D + 27]` that falls in the planting-out window of the crop (section 10). With no possible date, the crop is discarded.
2. **Proposed mode**:
   - if `indoor > 0` and `m − indoor ≥ today`: sowing under cover, with sowing = `m − indoor`;
   - if `indoor > 0` and that sowing would fall before today: plants, planted at `m`;
   - if `indoor = 0`: sowing in the open ground at `m`.

   `r` is computed as in section 2.
3. **Minimum harvest**: `min = ⌈duration ÷ 2⌉`. The crop is discarded if `r + min > F + 1`.
   - **Exception (winter crops)**: if the slot ends on 15 November (the end of the search window, so nothing closes it) and `r > F + 1` (the harvest only starts after it), the crop is kept and this step does not discard it.
4. **Proposed end**: `f = min(F + 1, r + duration)`. For a winter crop (exception of step 3), `f = r + duration`, possibly in the next year, with no end edited by hand. The score counts only the days up to `F + 1`. If `f < r + duration`, the planting is created with this end as an **end edited by hand**, so that it is not recomputed and stays in the slot.
5. **Filling score**: `(f − D) ÷ N`, between 0 and 1.

**Ranking**, at most 3 crops:
1. score rounded to the tenth, from largest to smallest;
2. then a family different from that of the last planting that ended before D on a segment of Z (a planting that ended before D has an end `f ≤ D`; if several end on the same day, all count);
3. then in alphabetical order of the **name shown** (in the language of the interface; the Rust core sorts with the English name and also sends the crops tied with the third one, so that the interface can sort them again and keep three).

The V1 catalogue has no green manure (S02). When it has some, they will always come last.

Tapping a suggestion opens the planting creation with the crop, the mode, the date, the zone Z and the proposed end already filled in.

Frost is not taken into account in V1 (S01, 2.2): the planting-out windows of the catalogue already do.

---

## 10. Planting-out window

Each crop has **a single window** (S01, F-CAT-2).

- **Window within one year** (`start ≤ end`): `[Y-start, Y-end]` for each year Y.
- **Window straddling two years** (`start > end`, garlic for example): `[Y-start, (Y+1)-end]`.

---

## 11. Reminders and tasks

- **Families of the 2 previous seasons** (bed panel): these are the families of the plantings of the bed whose occupation touches the years Y−1 or Y−2, where Y is the year of the cursor.
- **Tasks**: each planting produces one to three tasks, each with a type, a date and a **stable identifier** `<planting id>/<type>`, which does not change when the date moves.

  | Mode | Tasks (type · date) |
  |---|---|
  | Sowing under cover | `sow-indoor` · sowing, `transplant` · `m` |
  | Plants | `plant` · `m` ("Buy and plant") |
  | Open ground | `sow-ground` · `m` |
  | All | `harvest` · `r` ("Start harvesting") |

- **Task calendar** (PDF export): the tasks grouped by month.
- **After V1**, an iCal export (.ics) will make one event per task, with the stable identifier as `UID`. A calendar can then update its events instead of creating duplicates.

---

## 12. Test cases

The cases are written on the **reference catalogue** of S02 (section 3, 6 crops), not on the full V1 catalogue: otherwise each added crop would change the expected suggestions (T8 for example).

The data come from S02. T10 and T13 evaluate a crop for a slot (§9, steps 1 to 5), without going through the ranking. The bed is 1.2 m wide, with cells of 50 cm. Unless stated otherwise, today = 2026-11-14.

| # | Input | Expected |
|---|---|---|
| T1 | Tomato, sowing under cover on 2027-03-10 | `m` 2027-05-05 · `r` 2027-07-19 · `f` 2027-09-27 |
| T2 | Lettuce, open ground on 2027-04-01 | `m` 2027-04-01 · `r` 2027-06-18 (50 + 28 d) · `f` 2027-07-08 |
| T3 | Carrot, open ground on 2027-03-15 | `r` 2027-06-23 · `f` 2027-08-02 |
| T4 | Carrot, zone of 2 segments | 4 rows × 25 plants = ≈ 100 |
| T5 | Tomato, zone of 3 segments | 1 row × 2 plants = ≈ 2 |
| T6 | Tomato T1 on [1,3], lettuce plants on 2027-09-27 on [1,2] | No conflict: the tomato ends the day the lettuce starts |
| T7 | Bed of 6 segments, with only the carrot T3 on [1,6], year 2027 | Slots: [1,6] from 03-01 to 03-14 discarded (< 35 d); **[1,6] from 2027-08-02 to 2027-11-15** (106 d) |
| T8 | Suggestions for the slot of T7 | Lettuce only: sowing under cover on 2027-07-05, `m` 2027-08-02, `r` 2027-09-21, `f` 2027-10-11; the 5 other crops are out of their planting-out window |
| T9 | Carrot, sowing under cover on 2027-03-01, 21 days in pots entered | `m` 2027-03-22 · `r` 2027-06-30 · `f` 2027-08-09 |
| T10 | Slot [1,6] from 2027-05-01 to 2027-11-15, today = 2027-04-20 | The tomato is proposed **as plants** (its sowing under cover would fall on 2027-03-06, in the past): `m` 2027-05-01 · `r` 2027-07-15 · `f` 2027-09-23 |
| T11 | Bed of 6 segments: carrot T3 on [1,3], leek sown under cover on 2027-05-01 on [4,6]; bed deleted from 2028 | Leek: `m` 2027-07-10 · `r` 2027-11-07 · `f` 2028-02-05 → **deleted** (straddles 2028). Carrot kept |
| T12 | Plantings of T6; the lettuce is dragged by −1 day | Refused: the lettuce would start on 2027-09-26, in conflict with the tomato on [1,2] |
| T13 | Slot [1,6] from 2027-08-02 to 2027-10-05 (65 d) | Lettuce: `m` 2027-08-02 · `r` 2027-09-21 · `f` proposed 2027-10-06 instead of 2027-10-11, saved as an **end edited by hand** |

With an empty garden, the ranking of T10 gives leek (score 1.0), then carrot and zucchini (0.7): the tomato, with the same rounded score, comes after them in alphabetical order and is not among the 3 proposed.

---

## 13. Validation after a merge (lot 3)

In lot 3, the changes of several devices are merged (S05, §6). The merged document converges, but it may break the rules of the plan. After each merge, the app checks the merged document.

**Default behaviour, for all cases** (S05, §10): **nothing is deleted or corrected automatically**. Each inconsistency found is shown as "to resolve", and the user settles it.

| Case | Detection rule | Default behaviour |
|---|---|---|
| **Too many beds** | For a year Y, the layout of Y (section 7) holds more than 10 beds (S01, F-PLN-1) | All the beds of Y are kept. The year Y is flagged "to resolve", with the number of extra beds |
| **Orphan planting** | The bed of the planting does not exist in the layout of at least one year its occupation `[m, f[` touches | The planting is kept. It is flagged "to resolve" |
| **Overlap** | Two plantings of the same bed conflict by the rule of section 5 | Both plantings are kept. The pair is flagged "to resolve", with the common segments and period |
| **Past year changed** | The merge brings a change to the layout of a year Y, while Y is past on the date of the merge (Y < year of today) | The change is kept. The year Y is flagged "to resolve" |

The first three cases can be read on the state of the merged document. The fourth depends on what the merge brought, not only on the result (see question 5).

### After an import (lot 1)

Question 7 below is settled for the import, the only case of a received plan before lot 3. `read_import` and `import_garden` check the received document, on two levels:

| Level | Case | Behaviour |
|---|---|---|
| **Refusal** (`import_invalid`) | File that is not JSON, envelope or plan schema that cannot be read, invalid plot (dimensions, aisle, cell ≠ 25, 50, 100), bed length of zero or not a whole number of cells, a single invalid planting (zone, pots, manual end, empty occupation) | Nothing is read or replaced. A `format_version` newer than the app is refused separately (`import_too_recent`) |
| **Accepted, to resolve** | More than 10 beds in a year; orphan planting (unknown bed, or absent in an occupied year); zone beyond the bed; overlap of two plantings; bed outside the plot, overlapping another or too close (section 15); planting of a crop absent from the catalogue; note attached to an unknown bed or whose text or dates are invalid | The default behaviour above applies: **nothing is deleted or corrected**. `read_import` returns the list of inconsistencies, the interface shows it before the confirmation ("Replace anyway"), and the plan is imported as it is |

The criterion for refusal: what the calculations of `plan` cannot go through safely (division by a zero length, reversed zone). Everything else can be read on the state of the document and is settled afterwards with the usual actions. Past years are checked like the others: an imported file is not a change to a past year (the "Past year changed" case belongs to the merge). **The list stays visible after the import** (v0.9). `read_import` shows it at the confirmation; afterwards, `incoherences_plan` computes it again on the plan of the device, after the app loads and after each change. The interface shows it in a collapsible "To resolve: N" banner at the top of the screen, as long as the list is not empty. It empties by itself when each point is settled with the usual actions: no "Resolve" button, nothing is remembered (S05 §5 forbids storing derived data).

### Open questions

1. **Display**: settled for lot 1 (import): a collapsible banner, section 13 "After an import". Still open for the merge (lot 3): marker on the timeline and the garden view.
2. **Resolution**: which actions to offer to settle each case (delete, move, choose a version…)? Or does the user only fix with the usual actions?
3. **Blocking**: while an inconsistency exists, do the other actions stay possible? For example, can a planting be added on the 11th bed, or an overlapping bed be changed?
4. **Orphan planting**: where to show it, since its bed appears neither in the garden view nor in the timeline for the years concerned?
5. **Past year changed**:
   - Is a change made while the year was still current legitimate, or must it be flagged anyway?
   - Which date counts: that of the change on the originating device (its clock may be wrong), or that of the merge?
   - This case cannot be deduced from the state of the document. It needs the change history of the CRDT, or to store a trace, while S05 §5 forbids storing derived data.
   - How to resolve this case, since the editor does not allow changing a past year (S01, F-PLN-4)?
6. **Other possible cases**: should these cases be added to the list?
   - A bed is shortened on one device while the other adds a planting on its last segments. The zone then goes beyond the bed (`t2 > n`, section 4).
   - Two beds that overlap on the plan, or a bed that leaves the plot, after two moves made at the same time (S01, F-PLN-1).
7. **When to check**: settled for lot 1: at load and after each change, not only at the import (section 13 "After an import"). Still open for the merge.

---

## 14. Decisions

| Subject | Decision |
|---|---|
| Dates | Civil dates for the plan, UTC for technical timestamps (section 1) |
| Indoor mode for a crop with no pot phase | The user enters the time in pots (section 2) |
| Date stored in indoor mode | Always the sowing date (section 2) |
| Durations of a planting | Copied from the catalogue at creation (section 2) |
| Dragging a bar into a conflict | Refused, the bar goes back to its place (section 2) |
| Size of a segment | The cell size, fixed when the plot is created (section 4) |
| Width of the beds | Free to the centimetre; only the length follows the grid (section 4) |
| Deleting a bed | The plantings straddling the year of deletion are deleted (section 5) |
| Carrying changes of the plan forward | Until the next saved layout of the same bed (section 7) |
| Minimum harvest of the suggestions | Half the harvest duration (section 9) |
| Imported plan that breaks the rules | Refused if the calculations cannot read it; otherwise accepted as it is, with the list of inconsistencies to resolve (section 13) |
| Text of a period | 80 characters at most, like a note (section 16) |
| "Past" annotation | It ends before the year of today (section 16) |
| "Clear the plan" | The beds of the displayed year, their plantings and their notes from that year; the plot and the notes of the whole garden stay (section 16) |
| Sowing proposed in the past | Plants are proposed (section 9) |
| End shortened by a suggestion | It counts as an end edited by hand (section 9) |
| Changing the mode of a planting | Copies the time under cover from the catalogue again (section 2) |
| Bounds of the stages | Each range includes its lower bound (section 3) |
| Rounding of the shortcuts | Up for a half (section 4) |
| Planting "ended before D" | `f ≤ D` (section 9) |
| Deleting a bed | Also erases its layouts of the following years (section 5) |
| Frost and sunlight | Out of V1; the sunlight calculation is kept in annex A |
| Plot dimensions | 1 to 100 m per side, at least one cell (section 15) |
| Aisles | 30, 50 or 80 cm; a gap equal to the aisle is accepted, measured on one axis (section 15) |
| Bed at the edge of the plot | Allowed: the aisle is only required between beds (section 15) |
| Aisle too narrow | Refused like an overlap in v1, without a "Keep here" bubble (section 15) |
| Coordinates | Free in cm, the grid is an aid of the interface (section 15) |
| Check over several years | From the changed year to the last saved layout (section 15) |
| Name, minimum width | 1 to 30 characters, 10 cm (section 15) |
| "To resolve" list after an import | Collapsible banner, recomputed by `incoherences_plan` at load and after each change, empty when everything is settled (section 13) |

---

## 15. Plot and placement of the beds

As Tim is not available, these rules were settled while writing the editor (see the Decisions). They complete S01 F-TER-1, F-PLN-1 and F-PLN-2.

**Plot**
- Each side is **at least 1 m and at most 100 m**, and at least one cell.
- The aisle width is **30, 50 or 80 cm**.
- A changed plot (dimensions or aisle) is refused if, for any year of the garden, past ones included (the plot is one for the whole garden and a past layout cannot be repaired), a bed leaves the plot, covers another one or no longer keeps the aisle.

**Bed**
- The name is **1 to 30 characters** (leading and trailing spaces ignored).
- The width is **at least 10 cm**; the length stays a whole number of cells (section 7).
- The coordinates `(x, y)` are those of the top-left corner, in cm from the top-left corner of the plot. Rust imposes no grid: snapping is an aid of the interface.
- A bed **fits in the plot** if `x + length ≤ plot length` and `y + width ≤ plot width`. It may touch the edge: the aisle is only required between two beds.
- For two beds a and b, the gap on each axis is computed, `dx = max(b.x − (a.x + a.l), a.x − (b.x + b.l))` and `dy` likewise on y.
  - **Overlap**: `dx < 0` and `dy < 0`.
  - **Aisle too narrow**: no overlap, but `dx < aisle` and `dy < aisle`. A gap exactly equal to the aisle is accepted. The gap is measured on one axis, not diagonally.
- These rules are checked **for each year**, from the changed year to the last year that has a saved layout (a layout holds until the next one, section 7).
- A refusal changes nothing. Error codes: `plot_invalid`, `aisle_invalid`, `width_invalid`, `name_invalid`, `bed_outside_plot`, `bed_overlaps`, `aisle_too_narrow`.
- Refused position: in v1 the narrow aisle is refused like an overlap. The "Keep here" bubble of S01 F-PLN-2 is out of lot 1.

---

## Annex A — Sunlight (after V1)

This calculation is not done in V1 (S01, 2.2): it needs the orientation of the plot, the latitude and obstacles, which will come with it.

The number of **hours of direct sun** is counted at the centre of each cell, on **21 June** and **20 March**, then the **average** of the two days is taken. 20 March, lower, accounts for the long shadows of spring.

| Average hours of sun | Level |
|---|---|
| ≥ 6 h | Full sun |
| 3 – 6 h | Partial shade |
| < 3 h | Shade |

**Position of the sun**, computed every 15 minutes from 4 h to 20 h, in solar time:
- declination `δ = +23.44°` on 21 June, and `0°` on 20 March;
- hour angle `H = 15° × (t − 12)`, with t in hours;
- height `a`: `sin a = sin φ · sin δ + cos φ · cos δ · cos H`, where φ is the latitude (44.84° for Bordeaux);
- azimuth from the north `Az`: `cos Az = (sin δ − sin a · sin φ) ÷ (cos a · cos φ)`. `Az` is kept in the morning (H < 0) and `360° − Az` is taken in the afternoon;
- only the instants where `a > 5°` are counted, since a lower sun does not count for the crop.

**Cast shadow**: an obstacle is a block of height h, set on its footprint. The orientation of the plot serves to express `Az` in the axes of the grid. The point P is **in the shade** at a given instant if the segment on the ground that starts from P toward the sun, of length `h ÷ tan a`, cuts the footprint of an obstacle. Trees are treated as blocks. Beds cast no shadow.

**Sunlight of a bed**: it is the level of the majority of its cells, and in case of a tie the sunniest level.

---

## 16. Notes and periods

Rules checked by `crates/plan` before anything is saved (S01 F-VIS-5):

- **Past year read-only**: refused (`year_past`) if the displayed year is before the year of today, or if the annotation, to create or to change, ends before the year of today (a period that starts last year and ends this year is allowed). Changing, hiding or deleting an annotation that is already past is refused the same way.
- **Bed**: if given, it must exist in the plan (`bed_unknown`); it may no longer exist in the displayed year.
- **Text**: 1 to 80 characters once the spaces are removed, for a note as for a period (`text_invalid`).
- **Dates**: a period has an end, on the same day as its start or after (`annotation_dates_invalid`). A note has no end: if it is given one, it is ignored.
- **Hiding** is a change (`hidden`), subject to the same rules.
- **Clear the plan** (S01 F-PLN-4): from the displayed year (not a past year), deletes all the beds that exist that year or later (those created for a later year only included), with their plantings like `delete_bed` (section 5), and the annotations attached to these beds that end on 1 January of the year or after. The plot, the past years, the plantings already finished and the annotations of the whole garden are kept.

## 17. Clarifications of v0.14 (audit of the specs against the code)

- **Past years**: the layouts of a past year and its annotations are frozen; its **plantings** and the **bed names** stay editable, so that the history can be corrected (S01 F-PLA-4).
- **Shortening a bed** below a planting (section 4) is allowed after the warning (`plantings_exceeding`). The planting then overflows its bed: it appears in the "To resolve" list (section 13) and cannot be edited until it fits again.
- **Winter crops** (section 9, step 4): a suggestion that runs into the next year is also checked as in section 5 for that year. It is not proposed if it would conflict there, or if the bed has no layout then.
- **To resolve list** (section 13): **every** placement problem is listed (each bed outside the plot, each pair of beds too close), not only the first one.
- **Entered end** (section 2): an end entered by hand that equals the computed end is not a manual end.
- **Names**: bed names are trimmed of leading and trailing spaces when saved.
- **Limits**: years are accepted from 1900 to 2200 and durations (pots, time to harvest, harvest) up to 3,650 days; beyond, the error is `out_of_range`.
- **Straddling windows** (section 10): the rule exists for the schema, but no crop of the V1 catalogue uses a window that straddles two years.
