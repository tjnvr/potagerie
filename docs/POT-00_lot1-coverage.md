# Potagerie — Lot 1 (desktop) coverage

State of the requirements of S01 (§4) in the app at lot 1, read in the code on 15 November 2026. "Done": covered by the code. "Partial": part of the requirement is missing. "Postponed": not in lot 1. The commands are those of `src-tauri/src/commands.rs`.

| Requirement | State | Where | Remark |
|---|---|---|---|
| F-CPT-1 Sign-in | Done | not applicable | Removed: no account |
| F-CPT-2 Automatic saving | Done | header indicator, SQLite `Store` | Three states including "Saving failed" and "Retry". Multi-device merging belongs to lot 3 |
| F-CPT-3 Export and import | Done | Settings; `export_garden`, `read_import`, `import_garden`, `incoherences_plan` | "Older file" warning and "to resolve" list |
| F-CPT-4 Erase the data | Done | Settings; `erase_data` | Double confirmation |
| F-TER-1 Plot creation | Done | Plan, "Create my plot"; `create_plot`, `modify_plot` | Minimum aisle adjustable in the settings |
| F-PLN-1 Beds | Partial | Plan; `create_bed`, `modify_bed`, `delete_bed` | Four models (bed, square, mound, planter), limit of 10 beds; no warning for two beds with the same name |
| F-PLN-2 Aisles | Done | Plan and garden; `create_bed`, `move_bed`, `modify_bed` (`keep_close`) | The width is adjustable; a bed placed, moved or resized closer than the aisle asks "Keep here" or "Go back" (T01, S03 v0.18 §15) |
| F-PLN-4 A plan that evolves by season | Done | Plan, "read only" banner; `view_year`, `clear_plan` | Carry-over to the following years, past seasons read-only |
| F-PLN-5 Plan summary | Done | Plan, right-hand panel | Plot, workable, aisles, beds |
| F-PLN-6 Place and size | Partial | Plan; `move_bed` | Drag, handles and add by tap present; no rotate, no duplicate, no Undo / Redo (after lot 1) |
| F-CAT-1 Content | Done | `catalogue` | The crops of S02 |
| F-CAT-2 Crop sheet | Done | Catalogue, "Sheet" | Schema of S02 |
| F-CAT-3 Browsing | Done | Catalogue; `crops_of_season` | Filter by family and "in season now" |
| F-PLA-1 Create a planting | Done | Bed panel, timeline; `create_planting`, `preview_planting` | Computed dates, preview before validation |
| F-PLA-2 Edit or delete | Done | Timeline (click, drag); `modify_planting`, `shift_planting`, `delete_planting` | |
| F-PLA-3 Successions | Done | Timeline, planting creation | Segment conflict flagged with the period |
| F-PLA-4 Continuous time | Done | Garden, timeline | The diorama takes the layout of the cursor's year |
| F-PLA-5 Quantity calculation | Done | Planting creation ("≈ n plants, rows × per row") | Calculation in `crates/plan` |
| F-VIS-1 Isometric garden view | Done | Garden | Rotation, labels, "Free" signs, seasonal mood; zoom with the wheel, the trackpad or a pinch (×1 to ×4, no button) and panning by dragging the background (T02) |
| F-VIS-2 Timeline view | Done | Garden, under the diorama | Merged with the diorama; continuous scroll between years |
| F-VIS-3 Time cursor | Done | Garden | |
| F-VIS-4 Season playback | Done | Garden, ×1 and ×2 | Measured at 60 frames/s (10 beds, normal density) |
| F-VIS-5 Timeline annotations | Done | Timeline, "Annotate", double click / long press on a row, drag of a note or a period; bed panel, "Add a note"; `create_annotation`, `modify_annotation`, `delete_annotation` | Notes and periods, edit, delete, hide. A dragged note follows the row it is dropped on; a period, or a note of the whole garden, keeps its bed |
| F-VIS-6 Screen layout | Done | App shell | Checked at 360 and 1440 px |
| F-VIS-7 Bed panel | Partial | Garden and Plan | Read-only history present; the families of the two previous seasons are not shown (after lot 1). "Move" opens the plan editor with the bed selected (T04), disabled in a past season |
| F-CRE-1 Free slots | Done | Timeline (dotted), diorama ("Free") | |
| F-CRE-2 Suggestions | Done | Click on a slot | Up to 3 crops, pre-filled dates |
| F-EXP-1 Timeline export | Partial | Export screen; `export_timeline` | PDF (A4 only), PNG, period and beds of your choice; A3 after lot 1 |
| F-EXP-2 Content of the PDF and the image | Partial | Export preview | Phases, "today" marker, optional annotations; no header, plan thumbnail or task calendar (after lot 1) |

| State | Count |
|---|---|
| Done | 22 |
| Partial | 7 |
| Postponed | 0 |

The requirements specific to lot 2 (mobile) and lot 3 (multi-device synchronisation) are not in the table.
