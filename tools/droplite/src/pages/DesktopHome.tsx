import { useCallback, useEffect, useState } from "react";
import { getDesktopState, onReceivedItem, openReceiveFolder, refreshSession } from "../lib/api";
import type { DesktopState } from "../lib/types";
import { Header } from "../components/Header";
import { QRPanel } from "../components/QRPanel";
import { ReceiveList } from "../components/ReceiveList";
import { SecurityStatus } from "../components/SecurityStatus";
import { SessionTimer } from "../components/SessionTimer";
import { useI18n } from "../i18n";

export function DesktopHome() {
  const { t } = useI18n();
  const [state, setState] = useState<DesktopState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const load = useCallback(async () => {
    try {
      setState(await getDesktopState());
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t("failedToLoad"));
    }
  }, []);

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 5000);
    return () => window.clearInterval(timer);
  }, [load]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void onReceivedItem((item) => {
      setState((current) =>
        current
          ? {
              ...current,
              received: [item, ...current.received.filter((existing) => existing.id !== item.id)]
            }
          : current
      );
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      setState(await refreshSession());
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t("refreshFailed"));
    } finally {
      setRefreshing(false);
    }
  };

  if (!state) {
    return (
      <main className="shell centered">
        <p>{error ?? t("starting")}</p>
      </main>
    );
  }

  return (
    <main className="shell">
      <Header session={state.session} onRefresh={handleRefresh} refreshing={refreshing} />
      {error ? <div className="error-banner">{error}</div> : null}
      <div className="workspace">
        <div className="primary-column">
          <QRPanel session={state.session} />
          <div className="bottom-row">
            <SecurityStatus session={state.session} />
            <SessionTimer expiresAt={state.session.expires_at} />
          </div>
        </div>
        <ReceiveList items={state.received} onOpenFolder={() => void openReceiveFolder()} />
      </div>
    </main>
  );
}
