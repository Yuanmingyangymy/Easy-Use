import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { I18nProvider, addLangParam, normalizeLanguage, useI18n } from ".";

function Probe() {
  const { language, setLanguage, t } = useI18n();
  return (
    <div>
      <p>{language}</p>
      <p>{t("ready")}</p>
      <button onClick={() => setLanguage("zh-CN")}>zh</button>
    </div>
  );
}

describe("i18n", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("falls back to English for unknown languages", () => {
    expect(normalizeLanguage("fr-FR")).toBeNull();
  });

  it("updates text after switching language", () => {
    render(
      <I18nProvider>
        <Probe />
      </I18nProvider>
    );

    expect(screen.getByText("Ready to receive")).toBeInTheDocument();
    fireEvent.click(screen.getByText("zh"));
    expect(screen.getByText("准备接收")).toBeInTheDocument();
  });

  it("adds language to QR URLs", () => {
    expect(addLangParam("http://192.168.1.2:3000/?token=abc", "zh-CN")).toBe(
      "http://192.168.1.2:3000/?token=abc&lang=zh-CN"
    );
  });
});
