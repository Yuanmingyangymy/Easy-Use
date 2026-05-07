import { createContext, useContext, useMemo, useState, type ReactNode } from "react";
import { en } from "./locales/en";
import { zhCN } from "./locales/zh-CN";

export type Language = "en" | "zh-CN";
type TranslationKey = keyof typeof en;
type TranslationDictionary = Record<TranslationKey, string>;

const STORAGE_KEY = "droplite.language";

const dictionaries: Record<Language, TranslationDictionary> = {
  en,
  "zh-CN": zhCN
};

interface I18nContextValue {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: TranslationKey, values?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<Language>(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    return normalizeLanguage(stored) ?? detectLanguage();
  });

  const value = useMemo<I18nContextValue>(() => {
    const dictionary = dictionaries[language] ?? dictionaries.en;
    return {
      language,
      setLanguage: (nextLanguage) => {
        localStorage.setItem(STORAGE_KEY, nextLanguage);
        setLanguageState(nextLanguage);
      },
      t: (key, values) => interpolate(dictionary[key] ?? dictionaries.en[key], values)
    };
  }, [language]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n must be used inside I18nProvider");
  }
  return value;
}

export function detectLanguage(): Language {
  const language = navigator.language || navigator.languages?.[0] || "en";
  return normalizeLanguage(language) ?? "en";
}

export function normalizeLanguage(language?: string | null): Language | null {
  if (!language) return null;
  if (language.toLowerCase().startsWith("zh")) return "zh-CN";
  if (language.toLowerCase().startsWith("en")) return "en";
  return null;
}

export function addLangParam(url: string, language: Language): string {
  try {
    const parsed = new URL(url);
    parsed.searchParams.set("lang", language);
    return parsed.toString();
  } catch {
    return url;
  }
}

function interpolate(template: string, values?: Record<string, string | number>): string {
  if (!values) return template;
  return Object.entries(values).reduce(
    (result, [key, value]) => result.split(`{${key}}`).join(String(value)),
    template
  );
}
