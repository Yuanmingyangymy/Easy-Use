import { QrCode } from "lucide-react";
import { QRCodeCanvas } from "qrcode.react";
import type { SessionView } from "../lib/types";
import { formatBytes } from "../lib/format";

interface QRPanelProps {
  session: SessionView;
}

export function QRPanel({ session }: QRPanelProps) {
  return (
    <section className="qr-panel" aria-label="Connection QR code">
      <div className="qr-box">
        {session.connection_url ? (
          <QRCodeCanvas value={session.connection_url} size={220} level="M" includeMargin />
        ) : (
          <QrCode size={96} />
        )}
      </div>
      <div className="connection-details">
        <h2>Scan with your phone to drop files here</h2>
        <p className="url-line">{session.connection_url}</p>
        <dl>
          <div>
            <dt>Network</dt>
            <dd>Local network only</dd>
          </div>
          <div>
            <dt>Max file</dt>
            <dd>{formatBytes(session.max_upload_bytes)}</dd>
          </div>
          <div>
            <dt>Saved to</dt>
            <dd title={session.receive_dir}>{session.receive_dir}</dd>
          </div>
        </dl>
      </div>
    </section>
  );
}
