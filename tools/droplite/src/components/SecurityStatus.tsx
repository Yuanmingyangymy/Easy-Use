import { ShieldCheck } from "lucide-react";
import type { SessionView } from "../lib/types";
import { useI18n } from "../i18n";

interface SecurityStatusProps {
  session: SessionView;
}

export function SecurityStatus({ session }: SecurityStatusProps) {
  const { t } = useI18n();
  const note = session.local_ip === "127.0.0.1" ? t("noLanIp") : t("securityNote");

  return (
    <section className="security-panel" aria-label="Security status">
      <ShieldCheck size={20} />
      <div>
        <strong>{t("localNetworkOnly")}</strong>
        <p>{note}</p>
        <p className="connection-hint">{t("windowsHint")}</p>
      </div>
    </section>
  );
}
