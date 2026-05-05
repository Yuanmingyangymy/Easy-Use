import { RefreshCw } from "lucide-react";
import type { SessionView } from "../lib/types";

interface HeaderProps {
  session: SessionView;
  onRefresh: () => void;
  refreshing: boolean;
}

export function Header({ session, onRefresh, refreshing }: HeaderProps) {
  return (
    <header className="app-header">
      <div>
        <p className="eyebrow">Easy-Use / DropLite</p>
        <h1>DropLite</h1>
        <p className="status-text">Ready to receive</p>
      </div>
      <div className="header-actions">
        <div className="device-pill" title="Current device">
          {session.device_name}
        </div>
        <button className="icon-button labelled" onClick={onRefresh} disabled={refreshing} title="Refresh session">
          <RefreshCw size={18} />
          <span>Refresh session</span>
        </button>
      </div>
    </header>
  );
}
