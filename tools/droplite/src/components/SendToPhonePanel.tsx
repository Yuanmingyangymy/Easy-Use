import { AlertCircle, CheckCircle, File, FilePlus, Image as ImageIcon, Send, Type, Upload, Video } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { OutboxItem } from "../lib/types";
import { addOutboxFile, addOutboxText, chooseOutboxFiles, listOutboxItems } from "../lib/api";
import { formatBytes, formatIsoTime } from "../lib/format";
import { useI18n } from "../i18n";

type Notice = {
  kind: "ok" | "error";
  text: string;
};

export function SendToPhonePanel() {
  const { t } = useI18n();
  const [text, setText] = useState("");
  const [items, setItems] = useState<OutboxItem[]>([]);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [busy, setBusy] = useState(false);
  const [dragging, setDragging] = useState(false);
  const lastDropRef = useRef<{ key: string; at: number } | null>(null);

  const refreshOutbox = useCallback(async () => {
    setItems(await listOutboxItems());
  }, []);

  useEffect(() => {
    void refreshOutbox().catch(() => undefined);
  }, [refreshOutbox]);

  const handleSendText = async () => {
    const value = text.trim();
    if (!value) {
      setNotice({ kind: "error", text: t("emptyTextCannotBeSent") });
      return;
    }

    setBusy(true);
    try {
      const item = await addOutboxText(value);
      setItems((current) => upsertOutboxItem(current, item));
      setText("");
      setNotice({ kind: "ok", text: t("addedToPhoneOutbox") });
    } catch {
      setNotice({ kind: "error", text: t("failedToAddText") });
    } finally {
      setBusy(false);
    }
  };

  const handleChooseFiles = async () => {
    setBusy(true);
    try {
      const paths = uniquePaths(await chooseOutboxFiles());
      if (paths.length > 0) {
        await addFiles(paths);
      }
    } catch {
      setNotice({ kind: "error", text: t("failedToAddFile") });
    } finally {
      setBusy(false);
    }
  };

  const addFiles = useCallback(
    async (paths: string[]) => {
      const unique = uniquePaths(paths);
      if (unique.length === 0) return;

      setBusy(true);
      const added: OutboxItem[] = [];
      let failed = 0;

      for (const path of unique) {
        try {
          added.push(await addOutboxFile(path));
        } catch {
          failed += 1;
        }
      }

      if (added.length > 0) {
        setItems((current) => added.reduce(upsertOutboxItem, current));
      }

      if (failed > 0) {
        setNotice({ kind: "error", text: t("failedToAddFile") });
      } else {
        setNotice({ kind: "ok", text: t("addedToPhoneOutbox") });
      }
      setBusy(false);
    },
    [t]
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    try {
      void getCurrentWebview()
        .onDragDropEvent((event) => {
          if (event.payload.type === "enter" || event.payload.type === "over") {
            setDragging(true);
            return;
          }
          if (event.payload.type === "leave") {
            setDragging(false);
            return;
          }
          if (event.payload.type === "drop") {
            setDragging(false);
            const paths = uniquePaths(event.payload.paths);
            const dropKey = paths.join("\n");
            const now = Date.now();
            if (lastDropRef.current?.key === dropKey && now - lastDropRef.current.at < 750) {
              return;
            }
            lastDropRef.current = { key: dropKey, at: now };
            void addFiles(paths);
          }
        })
        .then((cleanup) => {
          if (cancelled) {
            cleanup();
            return;
          }
          unlisten = cleanup;
        })
        .catch(() => undefined);
    } catch {
      // Browser-only tests and Vite preview do not expose Tauri drag events.
    }

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [addFiles]);

  return (
    <section className="send-panel" aria-label={t("sendToPhone")}>
      <div className="section-heading">
        <div>
          <p className="eyebrow">{t("desktopToPhonePreview")}</p>
          <h2>{t("sendToPhone")}</h2>
        </div>
      </div>

      <p className="panel-note">{t("sendToPhoneDescription")}</p>
      <p className="panel-note muted-note">{t("mobileReceiveComingNext")}</p>

      {notice ? (
        <div className={`inline-status ${notice.kind}`} role={notice.kind === "error" ? "alert" : "status"}>
          {notice.kind === "ok" ? <CheckCircle size={16} /> : <AlertCircle size={16} />}
          <span>{notice.text}</span>
        </div>
      ) : null}

      <div className="send-form">
        <label htmlFor="send-to-phone-text">{t("typeTextToSendToPhone")}</label>
        <textarea
          id="send-to-phone-text"
          value={text}
          onChange={(event) => setText(event.target.value)}
          placeholder={t("typeTextToSendToPhone")}
        />
        <button className="icon-button labelled" type="button" onClick={handleSendText} disabled={busy}>
          <Send size={17} />
          <span>{t("sendText")}</span>
        </button>
      </div>

      <div className="send-file-row">
        <button className="text-button strong" type="button" onClick={handleChooseFiles} disabled={busy}>
          <FilePlus size={17} />
          <span>{t("chooseFiles")}</span>
        </button>
        <div className={`send-drop-zone${dragging ? " dragging" : ""}`} aria-label={t("dropFilesHereToSendToPhone")}>
          <Upload size={18} />
          <span>{t("dropFilesHereToSendToPhone")}</span>
        </div>
      </div>

      <div className="outbox-list-heading">
        <strong>{t("pendingForPhone")}</strong>
      </div>

      {items.length === 0 ? (
        <div className="outbox-empty">{t("waitingForPhoneToReceive")}</div>
      ) : (
        <ul className="outbox-list">
          {items.map((item) => (
            <li key={item.id} className="outbox-item">
              <OutboxIcon kind={item.kind} />
              <div className="outbox-copy">
                <div className="receive-title-row">
                  <strong>{item.kind === "text" ? item.content ?? t("receivedText") : item.displayName}</strong>
                  <span>{formatIsoTime(item.createdAt)}</span>
                </div>
                <p className="meta-line">
                  {labelForKind(item.kind, t)} {item.mimeType ? `· ${item.mimeType}` : ""}{" "}
                  {item.sizeBytes !== undefined ? `· ${formatBytes(item.sizeBytes)}` : ""}
                </p>
                <span className={`status-chip ${item.status}`}>{statusLabel(item.status, t)}</span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function upsertOutboxItem(items: OutboxItem[], item: OutboxItem): OutboxItem[] {
  return [item, ...items.filter((existing) => existing.id !== item.id)];
}

function uniquePaths(paths: string[]): string[] {
  return Array.from(new Set(paths.filter((path) => path.trim().length > 0)));
}

function OutboxIcon({ kind }: { kind: OutboxItem["kind"] }) {
  return (
    <div className="file-icon" aria-label={`${kind} outbox item`}>
      {kind === "text" ? (
        <Type size={22} />
      ) : kind === "image" ? (
        <ImageIcon size={22} />
      ) : kind === "video" ? (
        <Video size={22} />
      ) : (
        <File size={22} />
      )}
    </div>
  );
}

function labelForKind(kind: OutboxItem["kind"], t: ReturnType<typeof useI18n>["t"]) {
  if (kind === "text") return t("receivedText");
  if (kind === "image") return t("image");
  if (kind === "video") return t("video");
  return t("file");
}

function statusLabel(status: OutboxItem["status"], t: ReturnType<typeof useI18n>["t"]) {
  if (status === "downloaded") return t("downloaded");
  if (status === "expired") return t("sessionExpired");
  return t("waitingForPhoneToReceive");
}
