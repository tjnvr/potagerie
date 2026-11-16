// The final ranking of the suggestions of a slot (S03 §9). Rust ranks them by
// score, then family, then the English name, and sends the crops tied with the
// third one too: the last tie-break, the alphabetical order of the name, is made
// here with the names in the language of the interface.

import type { SuggestionView } from "./bindings";
import { nameCrop } from "./beds";
import { localeTag } from "./locale";

export const MAX_SUGGESTIONS = 3;

export function rankSuggestions(list: SuggestionView[]): SuggestionView[] {
  const collator = new Intl.Collator(localeTag(), { sensitivity: "base" });
  return [...list]
    .sort(
      (a, b) =>
        b.score_tenths - a.score_tenths ||
        Number(a.recent_family) - Number(b.recent_family) ||
        collator.compare(nameCrop(a.crop_id), nameCrop(b.crop_id)),
    )
    .slice(0, MAX_SUGGESTIONS);
}
