import { QrCode } from "lucide-react";
import { QRCodeCanvas } from "qrcode.react";
import type { SessionView } from "../lib/types";
import { formatBytes } from "../lib/format";
import { addLangParam, useI18n } from "../i18n";
import { ReceiveFolderControl } from "./ReceiveFolderControl";

interface QRPanelProps {
  session: SessionView;
  onReceiveDirectoryChange: (path: string) => void;
}

export function QRPanel({ session, onReceiveDirectoryChange }: QRPanelProps) {
  const { language, t } = useI18n();
  const connectionUrl = addLangParam(session.connection_url, language);
  const canScan = session.is_ready && session.port !== 0;

  return (
    <section className="qr-panel" aria-label="Connection QR code">
      <div className="qr-box">
        {canScan ? (
          <QRCodeCanvas value={connectionUrl} size={220} level="M" includeMargin />
        ) : (
          <QrCode size={96} />
        )}
      </div>
      <div className="connection-details">
        <h2>{canScan ? t("scanHint") : t("scanHintStarting")}</h2>
        <p className="url-line">{canScan ? connectionUrl : t("startingLocalServer")}</p>
        <dl>
          <div>
            <dt>{t("network")}</dt>
            <dd>{t("localNetworkOnly")}</dd>
          </div>
          <div>
            <dt>{t("maxFile")}</dt>
            <dd>{formatBytes(session.max_upload_bytes)}</dd>
          </div>
          <ReceiveFolderControl receiveDir={session.receive_dir} onChange={onReceiveDirectoryChange} />
        </dl>
      </div>
    </section>
  );
}
