// The tests expect the English messages (the base language), whatever the
// language of the machine that runs them.
import { overwriteGetLocale } from "./lib/paraglide/runtime.js";

overwriteGetLocale(() => "en");
