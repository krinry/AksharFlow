import { en, type Dictionary } from "./en";
import { hi } from "./hi";

export type UiLanguage = "en" | "hi";

export function getDictionary(lang: UiLanguage): Dictionary {
  return lang === "hi" ? hi : en;
}
