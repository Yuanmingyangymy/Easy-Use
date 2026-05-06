import { Check, Clipboard, File, FolderOpen, Image as ImageIcon, Video } from "lucide-react";
import { useState } from "react";
import type { ReceivedItem } from "../lib/types";
import { formatBytes, formatTime } from "../lib/format";
import { useI18n } from "../i18n";

interface ReceiveListProps {
  items: ReceivedItem[];
  onOpenFolder: () => void;
}

export function ReceiveList({ items, onOpenFolder }: ReceiveListProps) {
  const { t } = useI18n();

  return (
    <section className="receive-panel" aria-label="Received items">
      <div className="section-heading">
        <div>
          <p className="eyebrow">{t("inbox")}</p>
          <h2>{t("recentlyReceived")}</h2>
        </div>
        <button className="icon-button" onClick={onOpenFolder} title={t("openFolder")}>
          <FolderOpen size={18} />
        </button>
      </div>

      {items.length === 0 ? (
        <div className="empty-state">
          <Check size={24} />
          <p>{t("noAccountCloudHistory")}</p>
        </div>
      ) : (
        <ul className="receive-list">
          {items.map((item) => (
            <li key={item.id} className="receive-item">
              <Preview item={item} />
              <div className="receive-copy">
                <div className="receive-title-row">
                  <strong>{item.kind === "text" ? t("receivedText") : item.name}</strong>
                  <span>{formatTime(item.received_at)}</span>
                </div>
                {item.kind === "text" ? (
                  <p className="text-preview">{item.text}</p>
                ) : (
                  <p className="meta-line">
                    {labelForKind(item.kind, item.mime, t)} {formatBytes(item.size)}
                  </p>
                )}
                <div className="item-actions">
                  {item.kind === "text" && item.text ? (
                    <button className="text-button" onClick={() => void navigator.clipboard.writeText(item.text ?? "")}>
                      <Clipboard size={16} />
                      <span>{t("copyText")}</span>
                    </button>
                  ) : (
                    <button className="text-button" onClick={onOpenFolder}>
                      <FolderOpen size={16} />
                      <span>{t("openFolder")}</span>
                    </button>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function Preview({ item }: { item: ReceivedItem }) {
  const [failed, setFailed] = useState(false);

  if (item.kind === "image" && item.preview_url && !failed) {
    return <img className="thumb" src={item.preview_url} alt={`${item.name} thumbnail`} onError={() => setFailed(true)} />;
  }

  return (
    <div className="file-icon" aria-label={`${item.kind} preview`}>
      {item.kind === "image" ? <ImageIcon size={22} /> : item.kind === "video" ? <Video size={22} /> : <File size={22} />}
    </div>
  );
}

function labelForKind(kind: ReceivedItem["kind"], mime: string | undefined, t: ReturnType<typeof useI18n>["t"]): string {
  if (mime) return mime;
  if (kind === "image") return t("image");
  if (kind === "video") return t("video");
  return t("file");
}
