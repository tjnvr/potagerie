// Generated with the catalogue (S02 v0.3): for each crop without its own
// drawing, the drawn crop it looks like in the diorama and the icon, and its
// colour on the timeline.

export const APPEARANCES: Record<string, { lookalike: string; color: string }> = {
  "eggplant": { lookalike: "tomato", color: "#6B3A7A" },
  "bell-pepper": { lookalike: "tomato", color: "#D93B2B" },
  "chili-pepper": { lookalike: "tomato", color: "#B8241C" },
  "cherry-tomato": { lookalike: "tomato", color: "#F06A4F" },
  "potato": { lookalike: "green-bean", color: "#C9A66B" },
  "physalis": { lookalike: "tomato", color: "#E8A33D" },
  "romaine-lettuce": { lookalike: "lettuce", color: "#7DB548" },
  "curly-endive": { lookalike: "lettuce", color: "#A6CF6A" },
  "escarole": { lookalike: "lettuce", color: "#B5D67C" },
  "endive": { lookalike: "carrot", color: "#E3E0B0" },
  "artichoke": { lookalike: "zucchini", color: "#7A9A8C" },
  "salsify": { lookalike: "carrot", color: "#8C7A5B" },
  "jerusalem-artichoke": { lookalike: "green-bean", color: "#D8B04A" },
  "parsnip": { lookalike: "carrot", color: "#E8D9A8" },
  "celeriac": { lookalike: "carrot", color: "#BFCF9A" },
  "celery": { lookalike: "carrot", color: "#9CC27A" },
  "fennel": { lookalike: "carrot", color: "#CFE3A0" },
  "parsley": { lookalike: "carrot", color: "#3E8E3A" },
  "chervil": { lookalike: "carrot", color: "#78B060" },
  "coriander": { lookalike: "carrot", color: "#5EA052" },
  "dill": { lookalike: "carrot", color: "#A9C95A" },
  "shelling-bean": { lookalike: "green-bean", color: "#B88A4A" },
  "pole-bean": { lookalike: "green-bean", color: "#4F9A3A" },
  "pea": { lookalike: "green-bean", color: "#7CC46A" },
  "snow-pea": { lookalike: "green-bean", color: "#9AD27F" },
  "fava-bean": { lookalike: "green-bean", color: "#6E8F4E" },
  "cucumber": { lookalike: "zucchini", color: "#2F7A3E" },
  "gherkin": { lookalike: "zucchini", color: "#4A8A35" },
  "melon": { lookalike: "zucchini", color: "#E9B872" },
  "watermelon": { lookalike: "zucchini", color: "#2E6B3A" },
  "pumpkin": { lookalike: "zucchini", color: "#E3782A" },
  "red-kuri-squash": { lookalike: "zucchini", color: "#D9602A" },
  "butternut-squash": { lookalike: "zucchini", color: "#E8B04F" },
  "pattypan-squash": { lookalike: "zucchini", color: "#F2OF7A" },
  "onion": { lookalike: "leek", color: "#C98A4B" },
  "spring-onion": { lookalike: "leek", color: "#E9E4D0" },
  "shallot": { lookalike: "leek", color: "#B5654A" },
  "garlic": { lookalike: "leek", color: "#E6DCCB" },
  "chives": { lookalike: "leek", color: "#5C9E4F" },
  "cabbage": { lookalike: "lettuce", color: "#8FB8A0" },
  "cauliflower": { lookalike: "lettuce", color: "#EDE6C8" },
  "broccoli": { lookalike: "lettuce", color: "#3F7D4E" },
  "brussels-sprout": { lookalike: "lettuce", color: "#5E8F4A" },
  "kale": { lookalike: "lettuce", color: "#2F5E45" },
  "kohlrabi": { lookalike: "lettuce", color: "#A7C7A0" },
  "chinese-cabbage": { lookalike: "lettuce", color: "#C7OF9A" },
  "pak-choi": { lookalike: "lettuce", color: "#6FAE6A" },
  "radish": { lookalike: "carrot", color: "#E0445E" },
  "black-radish": { lookalike: "carrot", color: "#3A3A3A" },
  "turnip": { lookalike: "carrot", color: "#B486C2" },
  "rutabaga": { lookalike: "carrot", color: "#C9A46B" },
  "arugula": { lookalike: "lettuce", color: "#4E8A3A" },
  "beetroot": { lookalike: "carrot", color: "#8E2A4A" },
  "swiss-chard": { lookalike: "lettuce", color: "#C8D84A" },
  "spinach": { lookalike: "lettuce", color: "#2E6E3A" },
  "basil": { lookalike: "lettuce", color: "#4FA35A" },
  "lambs-lettuce": { lookalike: "lettuce", color: "#3F7A3A" },
  "sweet-corn": { lookalike: "leek", color: "#E8C547" },
  "sweet-potato": { lookalike: "zucchini", color: "#C56A3A" },
  "okra": { lookalike: "tomato", color: "#7FA34A" },
  "purslane": { lookalike: "lettuce", color: "#8DBA5A" },
};

/** The six crops drawn by D01 (plants2.py), whose ids name the symbols. */
const DRAWN = new Set(["tomato", "lettuce", "carrot", "green-bean", "zucchini", "leek"]);

/** The drawn crop that stands for `crop` (`green_bean` or `green-bean`). */
export function drawingOf(crop: string): string {
  const id = crop.replaceAll("_", "-");
  if (DRAWN.has(id)) return id;
  return APPEARANCES[id]?.lookalike ?? "lettuce";
}
