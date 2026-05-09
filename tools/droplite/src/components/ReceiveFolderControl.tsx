import { FolderOpen, RotateCcw, Settings } from "lucide-react";
import { useState } from "react";
import { chooseReceiveDirectory, openReceiveFolder, resetReceiveDirectory, setReceiveDirectory } from "../lib/api";
import { useI18n } from "../i18n";

interface ReceiveFolderControlProps {
  receiveDir: string;
  onChange: (path: string) => void;
}

type Notice = {
  kind: "ok" | "error";
  text: string;
};

export function ReceiveFolderControl({ receiveDir, onChange }: ReceiveFolderControlProps) {
  const { t } = useI18n();
  const [notice, setNotice] = useState<Notice | null>(null);
  const [busy, setBusy] = useState(false);

  const handleChange = async () => {
    setBusy(true);
    try {
      const selected = await chooseReceiveDirectory(t("chooseReceiveFolder"));
      if (!selected) {
        return;
      }

      const updated = await setReceiveDirectory(selected);
      onChange(updated);
      setNotice({ kind: "ok", text: t("receiveFolderUpdated") });
    } catch {
      setNotice({ kind: "error", text: t("failedToUpdateReceiveFolder") });
    } finally {
      setBusy(false);
    }
  };

  const handleReset = async () => {
    setBusy(true);
    try {
      const updated = await resetReceiveDirectory();
      onChange(updated);
      setNotice({ kind: "ok", text: t("receiveFolderResetToDefault") });
    } catch {
      setNotice({ kind: "error", text: t("failedToUpdateReceiveFolder") });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="receive-folder-control">
      <div>
        <dt>{t("savedTo")}</dt>
        <dd title={receiveDir}>{receiveDir}</dd>
        <p>{t("newFilesSavedHere")}</p>
      </div>
      <div className="folder-actions">
        <button className="text-button" type="button" onClick={handleChange} disabled={busy}>
          <Settings size={15} />
          <span>{t("changeFolder")}</span>
        </button>
        <button className="text-button" type="button" onClick={() => void openReceiveFolder()} disabled={busy}>
          <FolderOpen size={15} />
          <span>{t("openFolder")}</span>
        </button>
        <button className="text-button" type="button" onClick={handleReset} disabled={busy}>
          <RotateCcw size={15} />
          <span>{t("reset")}</span>
        </button>
      </div>
      {notice ? (
        <p className={`folder-notice ${notice.kind}`} role={notice.kind === "error" ? "alert" : "status"}>
          {notice.text}
        </p>
      ) : null}
    </div>
  );
}
