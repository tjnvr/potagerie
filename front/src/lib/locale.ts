// The BCP 47 tag the dates and numbers are formatted with: the language the
// messages are shown in (en.json is the base, fr.json its translation).
import { getLocale } from "./paraglide/runtime.js";

export function localeTag(): string {
  return getLocale() === "fr" ? "fr-FR" : "en-GB";
}
