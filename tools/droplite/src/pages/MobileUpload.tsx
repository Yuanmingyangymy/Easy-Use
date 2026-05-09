import { useCallback, useEffect, useMemo, useState } from "react";
import { useI18n } from "../i18n";
import { formatBytes, formatIsoTime } from "../lib/format";
import type { OutboxItem } from "../lib/types";

interface MobileUploadProps {
  poll?: boolean;
}

export function MobileUpload({ poll = true }: MobileUploadProps = {}) {
  const { t } = useI18n();
  const token = useMemo(() => new URLSearchParams(window.location.search).get("token") ?? "", []);
  const [items, setItems] = useState<OutboxItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [expired, setExpired] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [downloadingId, setDownloadingId] = useState<string | null>(null);

  const acknowledge = useCallback(
    async (id: string) => {
      const response = await fetch(`/api/outbox/${encodeURIComponent(id)}/ack?token=${encodeURIComponent(token)}`, {
        method: "POST"
      });

      if (response.status === 401) {
        setExpired(true);
        return;
      }

      if (!response.ok) return;
      setItems((current) =>
        current.map((item) => (item.id === id ? { ...item, status: "downloaded" as const } : item))
      );
    },
    [token]
  );

  const loadOutbox = useCallback(async () => {
    if (!token) {
      setExpired(true);
      return;
    }

    try {
      const response = await fetch(`/api/outbox?token=${encodeURIComponent(token)}`);
      if (response.status === 401) {
        setExpired(true);
        return;
      }
      if (!response.ok) throw new Error("outbox failed");
      const data = (await response.json()) as { items?: OutboxItem[] };
      setItems(data.items ?? []);
      setError(null);
    } catch {
      setError(t("failedToLoadDesktopItems"));
    }
  }, [token, t]);

  useEffect(() => {
    void loadOutbox();
    if (!poll) return;

    const timer = window.setInterval(() => {
      if (!document.hidden && !expired) void loadOutbox();
    }, 1500);

    return () => window.clearInterval(timer);
  }, [expired, loadOutbox, poll]);

  const handleCopy = async (item: OutboxItem) => {
    if (!item.content) return;

    await writeClipboardText(item.content);
    setCopiedId(item.id);
    await acknowledge(item.id);
  };

  const handleDownload = async (item: OutboxItem) => {
    setDownloadingId(item.id);
    try {
      await acknowledge(item.id);
    } finally {
      setDownloadingId(null);
    }
  };

  return (
    <main className="mobile-upload-card">
      <h1>{t("appName")}</h1>
      <section aria-label={t("sendToDesktop")}>
        <h2>{t("sendToDesktop")}</h2>
        <div className="choice-grid">
          <button type="button">{t("sendText")}</button>
          <button type="button">{t("sendPhoto")}</button>
          <button type="button">{t("sendVideo")}</button>
          <button type="button">{t("sendFile")}</button>
        </div>
        <input aria-label={t("selectPhotoFiles")} type="file" accept="image/*" multiple />
        <input aria-label={t("selectVideoFiles")} type="file" accept="video/*" multiple />
        <input aria-label={t("selectFiles")} type="file" multiple />
      </section>

      <section aria-label={t("receiveFromDesktop")}>
        <h2>{t("receiveFromDesktop")}</h2>
        {expired ? <p role="alert">{t("sessionExpired")}</p> : null}
        {error && !expired ? <p role="alert">{error}</p> : null}
        {items.length === 0 && !expired ? <p>{t("noItemsFromDesktopYet")}</p> : null}
        {items.length > 0 ? (
          <ul>
            {items.map((item) => (
              <li key={item.id}>
                <article>
                  <strong>{item.kind === "text" ? t("sentFromDesktop") : item.displayName}</strong>
                  <p>
                    {labelForItem(item, t)} {item.mimeType ? ` | ${item.mimeType}` : ""}{" "}
                    {item.sizeBytes !== undefined ? `| ${formatBytes(item.sizeBytes)}` : ""}
                    {item.createdAt ? ` | ${formatIsoTime(item.createdAt)}` : ""}
                  </p>
                  {item.kind === "text" ? (
                    <>
                      <p>{item.content}</p>
                      <button type="button" onClick={() => void handleCopy(item)}>
                        {copiedId === item.id ? t("copied") : t("copyText")}
                      </button>
                    </>
                  ) : (
                    <a
                      href={downloadUrl(item.id, token)}
                      download={item.displayName}
                      onClick={(event) => {
                        event.preventDefault();
                        void handleDownload(item);
                      }}
                    >
                      {downloadingId === item.id ? t("downloading") : t("download")}
                    </a>
                  )}
                </article>
              </li>
            ))}
          </ul>
        ) : null}
      </section>
    </main>
  );
}

function downloadUrl(id: string, token: string): string {
  return `/api/outbox/${encodeURIComponent(id)}/download?token=${encodeURIComponent(token)}`;
}

function labelForItem(item: OutboxItem, t: ReturnType<typeof useI18n>["t"]): string {
  if (item.kind === "image") return t("image");
  if (item.kind === "video") return t("video");
  if (item.kind === "file") return t("file");
  return t("receivedText");
}

async function writeClipboardText(text: string): Promise<void> {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand("copy");
  textarea.remove();
}
