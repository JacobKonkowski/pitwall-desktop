import { en, type MessageKey } from "./en";

const catalogs = { en } as const;

export type Locale = keyof typeof catalogs;

let locale: Locale = "en";

export function setLocale(next: Locale) {
  locale = next;
}

export function t(key: MessageKey): string {
  return catalogs[locale][key] ?? catalogs.en[key] ?? key;
}

export { en };
export type { MessageKey };
