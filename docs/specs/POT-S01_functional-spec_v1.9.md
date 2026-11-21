# Potagerie — V1 functional specification

> Version: 1.9 (draft) · Date: 2026-11-21 · Author: Tim · Domain: potagerie.garden
> v1.9 (T06) brings the warning for two beds with the same name into lot 1, at Tim's request (F-PLN-1). v1.8 (T18) brings the first **red fruits** forward from the wider catalogue, at Tim's request: raspberry and strawberry, planted once and kept year after year (F-CAT-1, F-CAT-2, F-PLA-1, F-VIS-1, F-VIS-2); the other red fruits and the reminder to renew a planting at the end of its lifespan follow (T19, T20). v1.7 settles the inconsistencies found by the audit (catalogue size, season selector, performance target, what comes after lot 1). v1.6 is the English translation of v1.5; no requirement changes. Stack and lots: [S05](POT-S05_technical-spec_v0.21.md). Rules and calculations: [S03](POT-S03_rules-and-calculations_v0.20.md). Catalogue: [S02](POT-S02_crop-catalogue_v0.5.md).
> v1.3: three clarifications revealed by D02 (additional screens), see §11.

---

## 1. Vision

Potagerie is an **installed app for computer and phone** to **design, plan and visualise a vegetable garden**. It has no server and no account: a single user, and the data stays on their devices.
You draw your plot, place your beds on it, then organise your crops over time. The garden is shown as a **small isometric diorama** that grows through the season.

Potagerie is a **planning tool**, not a game. Its look is soft, calming and pleasant to watch.

### 1.1 Name and domain

- **Potagerie**: a French-sounding name, easy to pronounce internationally, more distinctive than "Potager" alone, already used by several apps and brands.
- Target domain: **potagerie.garden**. Its availability remains to be checked before purchase. It does not host the app.

### 1.2 Goals

- Design the plan of a **future garden** from an empty plot.
- Plan a whole growing season, successions included.
- **Visualise** the garden at any date and watch the season go by.
- Get help to fill the empty beds (free slots and suggestions).
- Find your garden on all your devices: by export and import before lot 3, by synchronisation in lot 3.

### 1.3 Target user

An amateur gardener, alone, preparing a family garden (a few beds, 10 to 100 m²). They plan mostly in winter on a computer and check in spring and summer on a phone (lot 2).

---

## 2. Scope

### 2.1 In V1

| Area | Content |
|---|---|
| Data | No account, a single user; automatic local saving; export and import of the garden to move from one device to another; synchronisation between devices in lot 3 |
| Plot | Dimensions, cell size |
| Plan | A single plan per user: beds and squares, aisles |
| Catalogue | The 67 annual crops of S02 and the first red fruits (v1.8), pre-filled, not editable in V1 |
| Planning | Plantings, successions, automatic calculation of dates and quantities |
| Visualisation | Dated isometric garden view, continuous timeline (Gantt) over several years, time cursor, season playback |
| Help | Free slots and crop suggestions |
| Time | Continuous timeline from one year to the next, history kept, one plan layout per year |
| Export | Crop timeline as PDF and PNG image |

V1 is delivered in three lots (S05, §9):

| Lot | Content |
|---|---|
| **1 · Desktop** | Computer app (macOS, Windows). The phone layout is already done |
| **2 · Mobile** | Phone app (iOS, Android); transfer between devices by export and import |
| **3 · Sync** | The same garden on all devices, without a server |

### 2.2 Out of V1

- Tracking reality: actual dates, "sown", "harvested", automatic readjustment.
- Harvest log, quantities, photos, notes.
- Live weather, notifications and reminders.
- Sharing or collaboration, public garden.
- Several gardens or several plans (versions).
- Customisable catalogue: varieties, edited durations, added crops.
- Any game mechanic: points, levels, quests, unlocks.
- **Sunlight**: plot orientation, latitude, obstacles that cast shade (house, wall, hedge, tree, shed), sunlight map. The calculation is already described (S03, annex A).
- **Decor**: compost bin, water tank, greenhouse, path, bench.
- **Climate**: frost dates, frost periods in the timeline.
- **Alerts**: frost, planting-out period, rotation, companions, exposure.
- **Wider catalogue**: green manures and the other perennials (rhubarb, asparagus, perennial herbs), with exposure and good and bad neighbours. The red fruits were brought forward in v1.8 (F-CAT-1).
- **Season album**: a frozen image of the garden at its peak, kept each year.
- Timeline export as CSV.
- Calendar export as iCal (.ics). The design already prepares it (S03, §11).
- Several planting-out windows for the same crop.

---

## 3. Concepts and vocabulary

| Term | Definition |
|---|---|
| **Plot** | The whole available area, rectangular in V1, cut into a grid. |
| **Cell** | Unit of the grid, 25, 50 or 100 cm on a side. It is chosen when the plot is created and never changes afterwards. |
| **Plan** | The layout of the beds on the plot. **A single plan per user**, which evolves from season to season: each season keeps the layout it had. |
| **Bed** | A rectangular growing area: bed, square, mound or planter. |
| **Crop** | A sheet of the catalogue, for example "Tomato". |
| **Planting** | A crop placed on a zone of a bed, over a period. It is the central object of planning. |
| **Segment** | A portion of the length of a bed, the size of a cell. A zone is a run of contiguous segments. |
| **Season** | A calendar year. It does not cut the timeline, which is continuous: it only serves to date the layouts of the plan. |
| **Start mode** | The way a crop is started: sowing under cover, buying plants, or sowing in the open ground. |
| **Planting out** | The day the crop arrives in the bed: transplanting, planting of bought plants, or sowing in the open ground. |
| **Free slot** | A period during which a bed has no planting. |

---

## 4. Features

### 4.1 Data and synchronisation

**F-CPT-1 — Sign-in**: removed. The app has no account.

**F-CPT-2 — Saving and synchronisation**
- Each change is saved automatically on the device, with no "Save" button. The app always works without a network.
- A discreet indicator shows the state: "Saved", "Saving…", or, if the write fails (disk full, data folder unavailable…), "Saving failed" with a "Retry" button. The change stays displayed and editable until saving has succeeded, so that it is not lost (§8).
- **Lot 3**: changes made on several devices are merged (S05, §6). Additions made at the same time on two devices are all kept; on a same field, the last write wins. The inconsistencies that result are handled by the validation after a merge (S03, §13). An indicator shows the sync state.

**F-CPT-3 — Export and import of the garden**
- Full export of the garden into a file, both as a backup and to move from one device to another.
- Importing an export replaces the existing garden, after confirmation.
- Before lot 3, this is the **transfer between devices**. The import warns if the received garden is older than that of the device (S05, §6).

**F-CPT-4 — Erase the app data**
- Erases all the data of the garden on the device, after a double confirmation.

---

### 4.2 Plot

**F-TER-1 — Plot creation**
- Entry of the length and width in metres.
- Choice of the cell size: 25, 50 or 100 cm (50 cm by default). A text warns that it cannot be changed afterwards.
- Choice of the minimum aisle width (F-PLN-2): 30, 50 or 80 cm (50 cm by default). A starting value is needed for the plan editor (F-PLN-6), but it can be changed at any time in the settings (E10).

Obstacles (F-TER-2), sunlight map (F-TER-3) and climate (F-TER-4) are out of V1 (see 2.2).

---

### 4.3 Plan

**F-PLN-1 — Beds**
- **10 beds at most** per season. A "Beds 8 / 10" counter is visible in the editor, and the items of the tray are greyed out at 10, with a message that explains it.
- Bed types: **bed** in the open ground (1.2 × 3 m by default), **square** (1 × 1 m), **mound** (1.2 m wide), raised **planter** (1.2 × 2 m). The type only changes the look and the starting size: all are planned the same way.
- **Dimensions**: the length is a whole number of cells, since it is cut into segments. The width is free, to the centimetre, to keep the classic 1.2 m.
- Each bed has a **free name** (30 characters at most), for example "The tomatoes" or "Shed side". On creation it receives a default name ("Bed A", "Square B"…) that can be changed at any time, directly in the bed panel.
- The name is shown everywhere the bed appears: label in the garden view, timeline row, bed panel, planting creation and exports. Two beds may have the same name, but the app flags it: the panel of each of them says that another bed of the season has the same name. Nothing is refused.
- Each bed can be moved and resized (see F-PLN-6).
- Two beds cannot overlap. A bed cannot leave the plot.

**F-PLN-6 — Place and size the beds**

Everything is done in the **plan editor** (E3), in a top view on the grid. The app has **two modes**, always reachable through the "Garden" and "Plan" tabs at the top of the screen: "Plan" opens the editor, "Garden" goes back to the garden view (the "See the garden" button of the editor does the same).

The editor has **no tools to choose**: the beds are handled directly.

- **Add**: a **tray of items** under the plan offers Bed, Square, Mound and Planter. You grab an item and drop it on the plot: it arrives at its default size, receives a default name and stays selected. The "Beds 8 / 10" counter is shown on the tray, and its items are greyed out at 10.
- **Select**: tap or click a bed. Tapping the grass deselects.
- **Move**: drag the bed with the mouse, or with a finger after a long press on a phone.
- **Resize**: with the 8 handles of the selected bed, or by entering length and width in the panel. The length moves cell by cell; the width is set to the centimetre.
- **Rotate** by a quarter turn, **duplicate**, **delete**: from the bar that appears under the selected bed, or from the panel.
- **Right-hand panel**: it always shows the selected bed (name, type, length, width) and, with no selection, the plan summary (F-PLN-5).
- **Placement aids**:
  - snapping to the grid and to the edges of neighbouring beds, with a dotted guide;
  - live dimensions to the neighbours and to the edges of the plot;
  - original position in dotted lines during the drag.
- **Year**: the editor changes the plan of the cursor's year (see F-PLN-4). The year is recalled at the top of the editor ("Plan 2027").
- **Refused positions** (overlap, leaving the plot): the bed turns pale red and goes back to its place if dropped.
- The plantings and annotations **stay attached** to the bed when it is moved. If it is shortened, the plantings that no longer fit are flagged before validation.
- **Undo / Redo** for each action, with the buttons of the header.

**F-PLN-2 — Aisles**
- Adjustable minimum aisle width: 30, 50 or 80 cm.
- When placing, the app keeps this width between two beds. If the bed is dropped closer, a bubble offers "Keep here" or "Go back".

Decor (F-PLN-3) is out of V1 (see 2.2).

**F-PLN-4 — A single plan, that evolves by season**
- Each user has **one plot and one plan**. There are no versions and no active plan to choose.
- **Each year keeps its layout.** The editor shows the plan of the year where the time cursor is. Past years never change.
- **Carry-over to the following years**: the layout is only saved for the years in which it changes. A change to a bed made in year Y applies to Y and to the following years, **until the next year in which that same bed was changed**. The other beds are not touched.
- **Past seasons are read-only** in the editor: a banner says "Plan 2026, read only". To change, bring the cursor back to the current year or a future year ("Today" button).
- **Current season**: a bed can be moved or resized even if it carries crops. The current season takes the new layout directly, and the crops follow their bed.
- **New season**: it takes over the layout of the previous season.
- **Deleting a bed** removes it from the displayed year and the following ones. Its plantings that are entirely past stay visible in their seasons. Its plantings that touch the displayed year or the following ones, **including those straddling the previous year**, are deleted, after a confirmation that gives their number.
- To start from scratch: "Clear the plan", in the settings, after confirmation. Like any change, it applies to the current season and the following ones.

**F-PLN-5 — Plan summary**
- Total area, workable area, aisle area.
- Number of beds.

---

### 4.4 Crop catalogue

**F-CAT-1 — Content**
- **In V1, the 67 annual crops of S02** (section 3 bis): nightshades, cabbages, roots, squashes, legumes, salads, alliums, a few annual herbs.
- The V1 catalogue holds 67 annual crops (S02); green manures and the other perennials come after V1.
- **Red fruits** (v1.8, S02 §3 ter): raspberry and strawberry, then redcurrant, blackcurrant, gooseberry, thornless blackberry and blueberry (in a planter, T19). They are **perennials**: planted once, they stay on their part of the bed year after year, until the user ends them or until their usual lifespan (about 10 years for the raspberry, 3 for the strawberry).

**F-CAT-2 — Crop sheet**

The schema is **frozen for V1**; it is detailed in S02.

| Field | Example (tomato) | Use |
|---|---|---|
| Name | Tomato | Display |
| Botanical family | Solanaceae | Suggestions, rotation after V1 |
| Spacing between rows | 80 cm | Quantity calculation |
| Spacing along the row | 60 cm | Quantity calculation |
| Time under cover | 8 weeks | Calculation of the transplanting date |
| Time to harvest | 75 days after planting out | Phases |
| Harvest length | 70 days | Phases |
| Planting-out window | 1 May – 15 June | Ideal period, suggestions |
| Frost sensitive | Yes | Frost alerts after V1 |
| Illustrations | 4 stages | Garden view |
| Lifespan, harvest each year | Raspberry: 10 years, from 25 June | Red fruits only (v1.8): end of planting, harvest every year |

Exposure and good and bad neighbours will come with the wider catalogue.

- A crop has **a single** planting-out window.
- A window may straddle two years (a window such as 15 October – 15 March is allowed by the schema; no crop of the V1 catalogue uses it).

**F-CAT-3 — Browsing**
- List that can be filtered by family and by planting-out season ("in season now").
- Detailed read-only sheet.

---

### 4.5 Planning

**F-PLA-1 — Create a planting**
- From a bed (garden view or timeline) or from a suggested free slot.
- Three questions, in this order:
  1. **Which crop?** Chosen from the catalogue, with the crops in season highlighted.
  2. **How do you start?** Three cards, always all available, with no advice specific to each crop: the user chooses freely.
     - **I sow under cover**: sowing in pots, then transplanting to the garden;
     - **I buy plants**: garden-centre plants, planted directly;
     - **I sow in the open ground**: sowing directly in the bed.
  3. **When?** A **single date**, the one the user knows, whose question follows the mode: "When do you sow?" (under cover or open ground) or "When do you plant?" (plants). For sowing under cover, a link "I rather know the transplanting date" allows entering the other date: the app deduces and stores the sowing date.
- The **bed** is known in advance (you start from a bed or a free slot) and can still be changed.
  - **Zone of the bed**: the bed is cut into **segments** of the size of a cell along its length. You tap the segments occupied by the planting, which must be contiguous. Shortcuts select the whole bed, ½, ⅓ or ¼, starting from the first free segment. The chosen zone is also shown on a small diagram of the bed.
- Values computed and shown immediately:
  - A **mini-timeline** of the steps with their dates: pots (sowing under cover only), in the ground, harvest.
  - **Transplanting** date for a sowing under cover (sowing date plus time under cover).
  - **Start of harvest** (planting out plus time to harvest).
  - **End of planting** (start of harvest plus harvest length), editable. For a red fruit, the end of its usual lifespan (planting out plus 10 years for a raspberry); **editing this end is how the user ends the planting**, even before its first harvest.
  - **Estimated quantity**: number of rows × plants per row, from the length of the zone, the width of the bed and the spacings.
- A planting **keeps the durations of the catalogue** it had when created: a catalogue update never shifts the existing plantings.

**F-PLA-2 — Edit or delete a planting**
- All the entered values stay editable.
- If the planting-out date is changed, the end is recomputed, unless it had been edited by hand.
- In the timeline, you can **drag a bar** to shift the planting. The duration is kept. If the new position creates a conflict, the shift is refused and the bar goes back to its place.
- Deleting asks for a confirmation.

**F-PLA-3 — Successions**
- Several plantings can follow one another on the same bed.
- Two plantings can coexist on a bed if their **zones do not overlap** during the same period. Otherwise validation is impossible and the app shows the segments and the period in conflict.

**F-PLA-4 — Continuous time**
- There is no **required** season selector: the timeline scrolls continuously from one year to the next, and the plantings straddling two years (an autumn sowing harvested the next year) appear in one piece. The header offers the neighbouring years as a shortcut.
- The diorama shows the plan **of the year where the cursor is** (F-PLN-4). When the cursor crosses 1 January, it takes the layout of the new year.
- Past crops stay viewable and editable (the plan of the past years, however, is frozen). They feed the reminder of the families grown.

**F-PLA-5 — Quantity calculation**
- Reference length = the length of the zone (number of segments × size of a segment). Width = the width of the bed.
- Rows = width ÷ spacing between rows, rounded down, at least 1.
- Plants per row = length ÷ spacing along the row, rounded down, at least 1.
- The result is shown as an **estimate** ("≈ 21 plants, 1 row × 21").

---

### 4.6 Visualisation

The main screen presents two views **synchronised on one date**, the *time cursor*.

**F-VIS-1 — Garden view (isometric)**
- The garden, rendered as an isometric diorama set on a plain background, with the layout of the plan of the cursor's year.
- The plot and the beds are drawn in simple volume.
- Each planting in the ground at the date of the cursor is represented by plants **on its zone** of the bed.
- **4 visual stages** according to progress:

  | Stage | Progress since planting out |
  |---|---|
  | Sowing | 0 – 15 % of the time to harvest |
  | Young shoot | 15 – 50 % |
  | Adult | 50 – 100 % |
  | Harvest | Beyond 100 % (fruits or vegetables visible) |

- A **red fruit** (v1.8) is shown every year of its life: bare in winter, in leaf in spring and autumn, in fruit during each summer harvest (S03 §3).

- A planting sown under cover, not yet transplanted, is marked by a **small label** planted on the bed. A planting of bought plants appears directly at the "young shoot" stage.
- **Seasonal mood**: the colour of the grass and the sky follows the date (frost in winter, flowers in spring, golden tones in autumn).
- Navigation:
  - zoom by pinching (phone), wheel or trackpad (computer), with no button;
  - panning by dragging the background;
  - a "Rotate the view" button to turn the garden by a quarter turn, and see what the tall plants hide.
- The "Plan" tab, at the top of the screen, opens the plan editor (E3).
- A tap or click on a bed opens its detail panel.

**F-VIS-2 — Timeline view (Gantt)**
- **Continuous timeline**: you drag it (mouse or finger) to go back or forward in time, with no break between years. The year is recalled above the months, and the passage to 1 January is marked by a line.
- **Scale**: by default, 12 months are visible. You can zoom in to 3 months or out to 3 years, by pinching or with the wheel.
- One row per bed, sorted by its position on the plan. A bed that does not exist yet (or any more) over a period has its row greyed out over that period.
- One bar per planting, cut into visually distinct **phases**: pots (hatched, sowing under cover only), growth (muted colour), harvest (full colour). A planting of bought plants carries a small "plant" icon at the start of its bar. A red fruit (v1.8) is **one long bar** over its whole life, with its harvest in full colour each year.
- Plantings that overlap on the same bed are stacked on several sub-rows.
- In the background: month columns and the "today" marker.
- A tap or click on a bar opens the planting edit. A drag shifts the planting (F-PLA-2).
- On a phone (lot 2), the finger also scrolls the timeline: a bar is shifted after a **long press**, as in the editor.

**F-VIS-3 — Time cursor**
- A vertical marker on the timeline and a date selector under the garden view, the two linked.
- You can drag it, tap or click on the timeline, or move forward one week with the ‹ and › buttons on either side of the date.
- On opening, the cursor is on **today**. A "Today" button at the top of the screen brings it back at any time.

**F-VIS-4 — Season playback**
- The "▶ Play the season" button moves the cursor forward over 12 months from its position, in about fifteen seconds. The timeline scrolls with it.
- The plants grow and change stage with a **soft animation**: they come out of the ground and grow with a slight bounce.
- You can pause, resume, and choose the speed (×1, ×2).
- If the system asks to reduce motion, the playback moves in steps, with no transition.

**F-VIS-5 — Timeline annotations**
- The timeline can be annotated with two types of annotations:
  - **Note**: a marker at a date, with a short text (80 characters at most), for example "Mulch the tomatoes". It is attached to **a bed** (its row in the timeline) or to **the whole garden**.
  - **Period**: a named range of dates, for example "Holidays, 1 → 21 August" or "Away". It covers a bed or the whole garden, and is shown as a coloured band behind the bars.
- Creation:
  - "Annotate" button in the header of the timeline;
  - a double click (computer) or a long press (phone) on a row, at the wanted date;
  - from the bed panel ("Add a note").
- An annotation can be edited, moved by dragging it, and deleted.
- The annotations of a bed are also shown in its panel, sorted by date.
- An annotation can be hidden from the timeline without being deleted ("Show the annotations" box of the timeline).
- The annotations are part of the timeline export ("Annotations" option ticked by default).

**F-VIS-6 — Screen layout**
- **Main navigation**: two tabs at the top of the screen, "Garden" (E5) and "Plan" (E3).
- **Computer** (lot 1): the garden view takes the top of the screen, the timeline is below, expandable. A side panel shows the detail (bed, planting).
- **Phone** (lot 2): the garden view is full screen with the time cursor at the bottom. The timeline and the detail panel open as sliding sheets.
- The layout adapts to the screen width **from lot 1**, so that the move to the phone in lot 2 does not require redoing the interface.

**F-VIS-7 — Bed panel**
- Name (editable in place), dimensions, area.
- Notes of the bed, sorted by date.
- Plantings of the cursor's year, with their dates and quantities.
- Families grown the 2 previous seasons, a simple reminder for rotation.
- **History**: the plantings of past years on this bed (crop, dates, quantity), read-only, collapsible.
- Actions: "Plant here", move (shortcut to the plan editor, bed already selected), edit, delete.

---

### 4.7 Free slots and suggestions

No alerts in V1 (see 2.2). The app only helps to fill the empty beds.

**F-CRE-1 — Free slot detection**
- For each bed, the app spots the **periods without a planting longer than 35 days** between 1 March and 15 November of each year.
- In the timeline, a free slot appears dotted. In the garden view, a "+ Free" sign is planted on the bed.

**F-CRE-2 — Suggestions**
- For each slot, the app proposes **up to 3 crops**:
  - planting out possible within the first 4 weeks of the slot, in the ideal window of the crop;
  - time to harvest, plus a minimum harvest, compatible with the length of the slot.
- The proposed mode is sowing under cover if the crop has one. If the sowing date already falls in the past, the app proposes plants instead.
- Ranking: good filling of the slot first, then a family different from the previous crop on the bed.
- A tap on the slot opens the planting creation, with the suggestions at the top of the list and the dates pre-filled. If the proposed end is shortened to fit the slot, it counts as an end edited by hand: it is not recomputed.

---

### 4.8 Timeline export

**F-EXP-1 — Timeline export (Gantt)**
- Reachable from the "Export the timeline" button, in the header of the timeline.
- Formats:
  - **PDF**, to print and display in the garden shed: A4 or A3, landscape, one or several pages depending on the number of beds.
  - **PNG image**, to share or keep a record.
- Options:
  - **Period**: a free range of dates. By default, the period the timeline shows on screen when the export is opened, zoom and scroll included (3 years at most). If the timeline has not been displayed yet, the calendar year where the cursor is.
  - **Beds**: all, or a selection.
  - **Content**: whether or not to show the phases (under cover, growth, harvest), the "today" marker and the annotations.
- The exported rendering takes over the visual style of the app, adapted to print: light background, colours readable in black and white thanks to the phase patterns.

**F-EXP-2 — Content of the PDF and the image**
- Header: name of the garden, exported period, export date.
- The timeline: one row per bed, months in columns, one bar per planting with the name of the crop.
- Legend: phases and annotations.
- **Plan thumbnail**: the top view of the garden, with the name of the beds, to find each row on the plot.
- Option **"Task calendar"**: the month-by-month list, deduced from the plantings, of what to do: sow under cover, buy plants, sow in the open ground, transplant, start harvesting.

---

## 5. Visual direction

- **Soft isometric, close to low-poly**: simple rounded shapes, no realism. The garden looks like a small island set on a plain background.
- **Pastel, calming palette**: tender greens, warm earth, a sky whose tint changes with the season.
- **Minimal interface**: the garden at the centre, discreet controls, large touch zones, little text on screen.
- **Mouse and touch first**: everything is done by dragging, tapping or clicking. No action requires the keyboard, which only serves to enter text or values.
- **Growth animations**: the plants come out of the ground and grow with a slight bounce. A single strong animation, the season playback. The rest stays sober.
- **Illustrations per crop**: a recognisable silhouette at each of the 4 stages. It is the visual signature of the app.
- **Accessibility**: the phases of the timeline are told apart by their pattern as much as by their colour, and the "reduce motion" option is respected.

---

## 6. Screens

| # | Screen | Main content |
|---|---|---|
| E2 | Plot creation | Dimensions, cell size, aisle width |
| E3 | Plan editor | Top view, tray of beds to drag, 10-bed counter, panel of the selected bed, summary |
| E5 | **Garden** (main) | Garden view, time cursor, season playback, expandable continuous timeline, "Today" button |
| E6 | Bed panel | Detail, plantings, history, actions |
| E7 | Planting creation / edit | Crop, start mode, single date, mini-timeline of the steps, zone of the bed |
| E9 | Catalogue | Filterable list, crop sheets |
| E10 | Settings | Aisles, export and import of the garden, clear the plan, erase the data |
| E11 | Timeline export | Format, period, beds, display options, preview |

Screen E1 (Sign-in) is removed with the account.

### 6.1 Main flow (first use)

1. Plot creation: dimensions and cell size (E2).
2. Placing the beds by dragging them from the tray (E3).
3. Switching to the Garden view; dragging the timeline to the season to prepare (E5).
4. Adding the plantings, bed by bed (E6 → E7).
5. Filling the free slots from the suggestions (E5 → E7).
6. Playing the season to visualise the result (E5).

---

## 7. Data model (functional view)

```
Garden (a single one per device, no account)
 └─ Plot (1)
     ├─ dimensions, cell size (fixed), aisle width
     └─ Plan (a single one)
         ├─ Beds[]: name
         │    └─ Layouts[]: year, type, position, dimensions
         │       (only for the years in which the bed changes; 10 beds at most per season)
         ├─ Plantings[]: bed, crop, start mode (indoor | plants | open ground),
         │             sowing or planting date, zone (first and last segment),
         │             durations copied from the catalogue, end (if edited by hand)
         └─ Annotations[]: type (note | period), start, end (period), bed or whole garden, text, hidden (yes/no)

Catalogue (global, read-only)
 └─ Crops[]: see F-CAT-2
```

**Computed, not stored** values: transplanting date, start of harvest, end (if not edited by hand), quantities, free slots, suggestions, task calendar.

---

## 8. Non-functional requirements

- **Platforms**: installed app on computer, macOS and Windows (lot 1), then on phone, iOS and Android (lot 2). Layout adapted from 360 px wide, prepared from lot 1.
- **Performance**: the garden view stays smooth (60 frames per second targeted) during the season playback, for a garden of 10 beds (the limit) and the plantings they hold.
- **Responsiveness**: the computed values update instantly while typing.
- **Data**: automatic saving, and nothing must be lost if the app is closed.
- **Language**: the interface is available in English (base language) and French, picked from the language of the system. All the texts, catalogue names included, are meant to be translated: adding a language comes down to adding a translation file, without touching anything else.
- **Privacy**: the data stays on the devices of the user. Nothing is sent to a server. The export and the erasure are available at any time.

---

## 9. Open questions

None for now.

---

## 10. Ideas beyond V1

- Sunlight: orientation, latitude, obstacles, sunlight map, perhaps with a manual cell-by-cell setting to correct the calculation.
- Decor: compost bin, water tank, greenhouse, path, bench.
- Climate, frost dates and periods.
- Alerts: frost, planting-out period, rotation, companions, exposure.
- Calendar export as iCal (.ics), then reminders ("sow the tomatoes under cover this week").
- Season album.
- Tracking reality and automatic readjustment of the plan.
- Harvest log and production summary.
- Local weather and adjustment of the frost alerts.
- Wider catalogue (green manures and the other perennials), then customisable, varieties.
- After lot 1: rotate and duplicate a bed, Undo / Redo, the reminder of the families grown in the bed panel, the A3 paper size and the header, plan thumbnail and task calendar of the PDF.
- Read-only sharing of the garden.
- Several gardens or several plans (versions).

---

## 11. Decisions since v1.2

Clarifications revealed by the design of D02 (additional screens) and settled without waiting for Tim:

| Subject | Decision | Why |
|---|---|---|
| Aisle width at plot creation | F-TER-1 also asks for the aisle width (30, 50 or 80 cm, 50 cm by default), editable afterwards in the settings (E10) | F-PLN-2 and E10 (§6) make the aisle adjustable, but no value existed before the first visit to the editor (E3, F-PLN-6): E2 must set one at creation |
| Default export period (v1.5) | The visible period of the timeline, zoom and scroll included | You export what you see: the calendar year forced retyping the dates after a zoom or a scroll |
| "Saving failed" state | F-CPT-2 adds this third state to the indicator, with a "Retry" button; the change stays displayed until it succeeds | S05 §7 already foresees that a write (`export_garden`, and the regular saving) may return an error, but F-CPT-2 only described "Saved" and "Saving…". Without this state, the requirement "nothing must be lost" (§8) would have no screen to show it |
| History of the bed panel | F-VIS-7 adds a History section (plantings of past years, read-only, collapsible) and a "Move" action | The table of screens (§6, E6) already promised "history" and "actions" in the plan of "detail, plantings, actions", but F-VIS-7 only detailed the reminder of the families over 2 seasons and three actions without moving |
| Language (v1.6) | English is the base language of the code and of the messages; French is its translation. Dates and numbers follow the language | The repository is written in English; the French users keep a French interface |
