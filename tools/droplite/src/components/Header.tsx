import { RefreshCw } from "lucide-react";
import type { SessionView } from "../lib/types";
import { useI18n, type Language } from "../i18n";

interface HeaderProps {
  session: SessionView;
  onRefresh: () => void;
  refreshing: boolean;
}

export function Header({ session, onRefresh, refreshing }: HeaderProps) {
  const { language, setLanguage, t } = useI18n();

  return (
    <header className="app-header">
      <div>
        <p className="eyebrow">{t("appScope")}</p>
        <h1>{t("appName")}</h1>
        <p className="status-text">{t("ready")}</p>
      </div>
      <div className="header-actions">
        <label className="language-select">
          <span>{t("language")}</span>
          <select value={language} onChange={(event) => setLanguage(event.target.value as Language)}>
            <option value="en">{t("english")}</option>
            <option value="zh-CN">{t("chinese")}</option>
          </select>
        </label>
        <div className="device-pill" title={t("currentDevice")}>
          {session.device_name}
        </div>
        <button className="icon-button labelled" onClick={onRefresh} disabled={refreshing} title={t("refreshSession")}>
          <RefreshCw size={18} />
          <span>{t("refreshSession")}</span>
        </button>
      </div>
    </header>
  );
}
