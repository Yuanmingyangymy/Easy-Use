import { render as rtlRender, type RenderOptions } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, type Language } from "../i18n";

interface CustomRenderOptions extends Omit<RenderOptions, "wrapper"> {
  language?: Language;
}

const LANGUAGE_STORAGE_KEY = "droplite.language";

function render(ui: ReactElement, { language = "en", ...options }: CustomRenderOptions = {}) {
  localStorage.setItem(LANGUAGE_STORAGE_KEY, language);

  function Wrapper({ children }: { children: ReactNode }) {
    return <I18nProvider>{children}</I18nProvider>;
  }

  return rtlRender(ui, { wrapper: Wrapper, ...options });
}

export * from "@testing-library/react";
export { render };
