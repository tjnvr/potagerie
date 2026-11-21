# Potagerie — Crop catalogue

> Version: 0.4 (draft) · Date: 15 November 2026 · Author: Tim
> v0.4: English translation of v0.3 (field names and crop ids follow the code). The catalogue does not change.

---

## 1. Scope

- **Reference catalogue: the 6 crops of section 3**, chosen to cover the different cases: long crop, fast crop, sowing in the open ground, big plant, crop straddling two years. The test cases of S03 §12 are written on this catalogue.
- **V1 catalogue: 67 annual crops** (section 3 bis), which include the 6 reference crops. Perennials (rhubarb, asparagus, strawberry, perennial herbs) stay out of V1: growing them over several years does not fit the schema.
- **The sheet schema (section 2) is frozen for V1.** It only holds the fields the app needs to **calculate**: timeline phases, quantities, free slots and suggestions.
- Out of V1: good and bad neighbours, exposure, green manures, descriptive texts, varieties. They come with the growth of the catalogue.
- The values are **indicative**, to refine with use.

---

## 2. Sheet

| Field | Type | Unit | Rule |
|---|---|---|---|
| `id` | text | — | Stable identifier, lowercase ASCII with hyphens (`tomato`) |
| `name` | text | — | English name ("Tomato"); the names shown to the user come from the message files |
| `family` | text | — | Botanical family, to rank the suggestions |
| `indoor_days` | integer | days | Time in pots before transplanting. 0 if the crop is not usually sown under cover |
| `harvest_after_days` | integer | days | Time from planting out to the start of the harvest |
| `harvest_duration_days` | integer | days | Length of the harvest period |
| `planting_out` | period | `MM-DD` → `MM-DD` | Ideal planting-out window, a single one per crop. May straddle two years (start > end) |
| `row_cm` | integer | cm | Spacing between rows |
| `spacing_cm` | integer | cm | Spacing along the row |
| `frost_sensitive` | boolean | — | Not used in V1; will serve the frost alerts |

The **4 illustrations** (sowing, young shoot, adult, harvest) are tied to the `id` and are not part of the data.

---

## 3. The 6 reference crops

| Crop | Family | Indoor | Harvest after | Harvest length | Planting out | Row × spacing | Frost |
|---|---|---|---|---|---|---|---|
| Tomato | Solanaceae | 56 d | 75 d | 70 d | 05-01 → 06-15 | 80 × 60 cm | sensitive |
| Lettuce | Asteraceae | 28 d | 50 d | 20 d | 03-01 → 09-30 | 30 × 30 cm | hardy |
| Carrot | Apiaceae | 0 | 100 d | 40 d | 03-01 → 07-15 | 25 × 4 cm | hardy |
| Green bean | Fabaceae | 0 | 60 d | 30 d | 05-10 → 07-31 | 50 × 10 cm | sensitive |
| Zucchini | Cucurbitaceae | 21 d | 50 d | 80 d | 05-01 → 06-30 | 100 × 100 cm | sensitive |
| Leek | Amaryllidaceae | 70 d | 120 d | 90 d | 05-15 → 07-31 | 30 × 12 cm | hardy |

## 3 bis. Full V1 catalogue

This table is generated from `crates/catalogue/crops.json`, which is the reference. For crops sown in the open ground (indoor 0), "harvest after" counts from the sowing.

| id | Name | Family | Indoor (d) | Harvest after (d) | Harvest length (d) | Planting out | Row × spacing (cm) | Frost |
|---|---|---|---|---|---|---|---|---|
| `tomato` | Tomato | Solanaceae | 56 | 75 | 70 | 05-01 → 06-15 | 80 × 60 | yes |
| `lettuce` | Lettuce | Asteraceae | 28 | 50 | 20 | 03-01 → 09-30 | 30 × 30 | no |
| `carrot` | Carrot | Apiaceae | 0 | 100 | 40 | 03-01 → 07-15 | 25 × 4 | no |
| `green-bean` | Green bean | Fabaceae | 0 | 60 | 30 | 05-10 → 07-31 | 50 × 10 | yes |
| `zucchini` | Zucchini | Cucurbitaceae | 21 | 50 | 80 | 05-01 → 06-30 | 100 × 100 | yes |
| `leek` | Leek | Amaryllidaceae | 70 | 120 | 90 | 05-15 → 07-31 | 30 × 12 | no |
| `eggplant` | Eggplant | Solanaceae | 70 | 75 | 70 | 05-15 → 06-15 | 70 × 50 | yes |
| `bell-pepper` | Bell pepper | Solanaceae | 70 | 80 | 70 | 05-15 → 06-15 | 60 × 45 | yes |
| `chili-pepper` | Chili pepper | Solanaceae | 70 | 80 | 70 | 05-15 → 06-15 | 50 × 40 | yes |
| `cherry-tomato` | Cherry tomato | Solanaceae | 56 | 65 | 80 | 05-01 → 06-15 | 80 × 50 | yes |
| `potato` | Potato | Solanaceae | 0 | 100 | 30 | 03-20 → 05-15 | 70 × 35 | yes |
| `physalis` | Physalis | Solanaceae | 60 | 90 | 60 | 05-15 → 06-15 | 80 × 60 | yes |
| `romaine-lettuce` | Romaine lettuce | Asteraceae | 28 | 60 | 20 | 03-15 → 08-31 | 35 × 30 | no |
| `curly-endive` | Curly endive | Asteraceae | 30 | 70 | 30 | 06-01 → 08-15 | 35 × 35 | no |
| `escarole` | Escarole | Asteraceae | 30 | 75 | 30 | 06-01 → 08-15 | 35 × 35 | no |
| `endive` | Endive | Asteraceae | 0 | 150 | 30 | 05-15 → 06-30 | 30 × 10 | no |
| `artichoke` | Artichoke | Asteraceae | 60 | 150 | 60 | 04-15 → 05-31 | 100 × 100 | yes |
| `salsify` | Salsify | Asteraceae | 0 | 180 | 90 | 03-15 → 05-15 | 30 × 8 | no |
| `jerusalem-artichoke` | Jerusalem artichoke | Asteraceae | 0 | 200 | 90 | 03-01 → 04-30 | 80 × 40 | no |
| `parsnip` | Parsnip | Apiaceae | 0 | 140 | 90 | 03-15 → 05-31 | 35 × 10 | no |
| `celeriac` | Celeriac | Apiaceae | 70 | 140 | 60 | 05-15 → 06-15 | 50 × 40 | no |
| `celery` | Celery | Apiaceae | 70 | 110 | 60 | 05-15 → 06-30 | 50 × 35 | no |
| `fennel` | Fennel | Apiaceae | 30 | 80 | 30 | 06-01 → 07-31 | 40 × 25 | no |
| `parsley` | Parsley | Apiaceae | 0 | 75 | 150 | 03-15 → 07-31 | 25 × 10 | no |
| `chervil` | Chervil | Apiaceae | 0 | 50 | 60 | 03-01 → 09-30 | 25 × 10 | no |
| `coriander` | Coriander | Apiaceae | 0 | 45 | 30 | 04-01 → 08-31 | 25 × 10 | no |
| `dill` | Dill | Apiaceae | 0 | 55 | 40 | 04-15 → 07-31 | 30 × 20 | no |
| `shelling-bean` | Shelling bean | Fabaceae | 0 | 90 | 30 | 05-10 → 06-30 | 50 × 10 | yes |
| `pole-bean` | Pole bean | Fabaceae | 0 | 70 | 60 | 05-15 → 07-15 | 80 × 15 | yes |
| `pea` | Pea | Fabaceae | 0 | 80 | 30 | 02-15 → 04-30 | 50 × 5 | no |
| `snow-pea` | Snow pea | Fabaceae | 0 | 75 | 30 | 02-15 → 04-30 | 50 × 5 | no |
| `fava-bean` | Fava bean | Fabaceae | 0 | 110 | 30 | 02-01 → 04-15 | 50 × 15 | no |
| `cucumber` | Cucumber | Cucurbitaceae | 28 | 55 | 60 | 05-15 → 06-30 | 100 × 50 | yes |
| `gherkin` | Gherkin | Cucurbitaceae | 28 | 55 | 60 | 05-15 → 06-30 | 100 × 50 | yes |
| `melon` | Melon | Cucurbitaceae | 30 | 80 | 40 | 05-15 → 06-15 | 100 × 80 | yes |
| `watermelon` | Watermelon | Cucurbitaceae | 30 | 90 | 30 | 05-20 → 06-15 | 150 × 100 | yes |
| `pumpkin` | Pumpkin | Cucurbitaceae | 28 | 110 | 30 | 05-15 → 06-15 | 200 × 150 | yes |
| `red-kuri-squash` | Red kuri squash | Cucurbitaceae | 28 | 100 | 30 | 05-15 → 06-15 | 150 × 100 | yes |
| `butternut-squash` | Butternut squash | Cucurbitaceae | 28 | 110 | 30 | 05-15 → 06-15 | 150 × 100 | yes |
| `pattypan-squash` | Pattypan squash | Cucurbitaceae | 21 | 55 | 70 | 05-15 → 06-30 | 100 × 100 | yes |
| `onion` | Onion | Amaryllidaceae | 0 | 120 | 30 | 03-01 → 04-15 | 25 × 10 | no |
| `spring-onion` | Spring onion | Amaryllidaceae | 0 | 180 | 60 | 08-15 → 09-30 | 25 × 8 | no |
| `shallot` | Shallot | Amaryllidaceae | 0 | 110 | 30 | 02-15 → 03-31 | 25 × 15 | no |
| `garlic` | Garlic | Amaryllidaceae | 0 | 220 | 30 | 10-15 → 12-15 | 25 × 12 | no |
| `chives` | Chives | Amaryllidaceae | 50 | 60 | 150 | 04-01 → 06-30 | 30 × 25 | no |
| `cabbage` | Cabbage | Brassicaceae | 35 | 90 | 40 | 05-01 → 07-15 | 60 × 50 | no |
| `cauliflower` | Cauliflower | Brassicaceae | 35 | 90 | 20 | 05-15 → 07-15 | 60 × 60 | no |
| `broccoli` | Broccoli | Brassicaceae | 35 | 75 | 30 | 05-01 → 07-15 | 60 × 50 | no |
| `brussels-sprout` | Brussels sprout | Brassicaceae | 40 | 130 | 90 | 05-15 → 06-30 | 70 × 60 | no |
| `kale` | Kale | Brassicaceae | 35 | 70 | 120 | 05-01 → 07-31 | 60 × 50 | no |
| `kohlrabi` | Kohlrabi | Brassicaceae | 30 | 55 | 20 | 04-01 → 08-15 | 35 × 25 | no |
| `chinese-cabbage` | Chinese cabbage | Brassicaceae | 0 | 70 | 30 | 07-15 → 08-31 | 40 × 35 | no |
| `pak-choi` | Pak choi | Brassicaceae | 0 | 50 | 20 | 07-15 → 09-15 | 30 × 25 | no |
| `radish` | Radish | Brassicaceae | 0 | 28 | 14 | 03-01 → 09-30 | 15 × 3 | no |
| `black-radish` | Black radish | Brassicaceae | 0 | 90 | 60 | 07-01 → 08-15 | 30 × 10 | no |
| `turnip` | Turnip | Brassicaceae | 0 | 60 | 30 | 03-01 → 09-15 | 25 × 10 | no |
| `rutabaga` | Rutabaga | Brassicaceae | 0 | 110 | 60 | 06-01 → 07-15 | 40 × 20 | no |
| `arugula` | Arugula | Brassicaceae | 0 | 35 | 30 | 03-15 → 09-30 | 20 × 5 | no |
| `beetroot` | Beetroot | Amaranthaceae | 0 | 90 | 60 | 04-15 → 07-15 | 30 × 10 | no |
| `swiss-chard` | Swiss chard | Amaranthaceae | 0 | 70 | 120 | 04-15 → 07-15 | 40 × 30 | no |
| `spinach` | Spinach | Amaranthaceae | 0 | 50 | 30 | 03-01 → 09-30 | 25 × 10 | no |
| `basil` | Basil | Lamiaceae | 42 | 40 | 90 | 05-15 → 06-30 | 30 × 25 | yes |
| `lambs-lettuce` | Lamb's lettuce | Valerianaceae | 0 | 70 | 90 | 07-15 → 09-30 | 15 × 8 | no |
| `sweet-corn` | Sweet corn | Poaceae | 21 | 90 | 20 | 05-15 → 06-15 | 70 × 30 | yes |
| `sweet-potato` | Sweet potato | Convolvulaceae | 0 | 120 | 20 | 05-20 → 06-15 | 80 × 40 | yes |
| `okra` | Okra | Malvaceae | 42 | 70 | 60 | 05-25 → 06-20 | 70 × 40 | yes |
| `purslane` | Purslane | Portulacaceae | 0 | 50 | 60 | 05-15 → 07-31 | 25 × 15 | yes |

For the crops without their own drawing, the diorama and the icons reuse the drawing of a close crop (`front/src/lib/appearances.ts`), and the timeline gives them a colour of their own.

---

## 4. Data format

A `crops.json` file, loaded as is by the app:

```json
[
  {
    "id": "tomato",
    "name": "Tomato",
    "family": "Solanaceae",
    "indoor_days": 56,
    "harvest_after_days": 75,
    "harvest_duration_days": 70,
    "planting_out": { "start": "05-01", "end": "06-15" },
    "row_cm": 80,
    "spacing_cm": 60,
    "frost_sensitive": true
  }
]
```

---

## 5. Open questions

1. Which crops to add after V1, and in which order? To see later.

## 6. Decisions

| Subject | Decision |
|---|---|
| V1 catalogue | 67 annual crops (section 3 bis), including the 6 reference crops (section 3), on which the cases of S03 are written |
| Perennials | Out of V1 |
| Families | 14 botanical families; Amaranthaceae for beetroot, chard and spinach (current classification) |
| Sheet schema | Frozen for V1 (section 2) |
| Several planting-out windows | No: a single window per crop |
| Language | The catalogue is in English; the names shown come from the message files (`crop_<id>` keys) |
