import { Check, Clipboard, File, FolderOpen, Image as ImageIcon } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ReceivedItem } from "../lib/types";
import { formatBytes, formatTime } from "../lib/format";

interface ReceiveListProps {
  items: ReceivedItem[];
  onOpenFolder: () => void;
}

export function ReceiveList({ items, onOpenFolder }: ReceiveListProps) {
  return (
    <section className="receive-panel" aria-label="Received items">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Inbox</p>
          <h2>Recently received</h2>
        </div>
        <button className="icon-button" onClick={onOpenFolder} title="Open folder">
          <FolderOpen size={18} />
        </button>
      </div>

      {items.length === 0 ? (
        <div className="empty-state">
          <Check size={24} />
          <p>No account. No cloud. No history.</p>
        </div>
      ) : (
        <ul className="receive-list">
          {items.map((item) => (
            <li key={item.id} className="receive-item">
              <Preview item={item} />
              <div className="receive-copy">
                <div className="receive-title-row">
                  <strong>{item.kind === "text" ? "Received text" : item.name}</strong>
                  <span>{formatTime(item.received_at)}</span>
                </div>
                {item.kind === "text" ? (
                  <p className="text-preview">{item.text}</p>
                ) : (
                  <p className="meta-line">
                    {item.mime ?? "File"} {formatBytes(item.size)}
                  </p>
                )}
                <div className="item-actions">
                  {item.kind === "text" && item.text ? (
                    <button className="text-button" onClick={() => void navigator.clipboard.writeText(item.text ?? "")}>
                      <Clipboard size={16} />
                      <span>Copy text</span>
                    </button>
                  ) : (
                    <button className="text-button" onClick={onOpenFolder}>
                      <FolderOpen size={16} />
                      <span>Open folder</span>
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
  if (item.kind === "image" && item.path) {
    return <img className="thumb" src={convertFileSrc(item.path)} alt="" />;
  }

  return (
    <div className="file-icon">
      {item.kind === "image" ? <ImageIcon size={22} /> : <File size={22} />}
    </div>
  );
}
