import { useI18n } from "../i18n";

export function MobileUpload() {
  const { t } = useI18n();

  return (
    <main className="mobile-upload-card">
      <h1>{t("appName")}</h1>
      <div className="choice-grid">
        <button type="button">{t("sendText")}</button>
        <button type="button">{t("sendPhoto")}</button>
        <button type="button">{t("sendVideo")}</button>
        <button type="button">{t("sendFile")}</button>
      </div>
      <input aria-label={t("sendPhoto")} type="file" accept="image/*" multiple />
      <input aria-label={t("sendVideo")} type="file" accept="video/*" multiple />
      <input aria-label={t("sendFile")} type="file" multiple />
    </main>
  );
}
