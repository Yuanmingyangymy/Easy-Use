import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  const [activeTab, setActiveTab] = useState<"send" | "receive">("send");
  const [downloadFallback, setDownloadFallback] = useState<{ id: string; url: string } | null>(null);
  const activeTabRef = useRef<"send" | "receive">("send");
  const previousItemIdsRef = useRef<Set<string>>(new Set());
  const [hasNewDesktopItem, setHasNewDesktopItem] = useState(false);
  const restrictedBrowser = isRestrictedDownloadBrowser();
  const hasDownloadableItems = items.some((item) => item.kind !== "text");

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
      const uniqueItems = dedupeOutboxItems(data.items ?? []);
      const previous = previousItemIdsRef.current;
      const hasNew = uniqueItems.some((item) => !previous.has(item.id));
      previousItemIdsRef.current = new Set(uniqueItems.map((item) => item.id));
      if (hasNew && activeTabRef.current === "send") {
        setHasNewDesktopItem(true);
      }
      setItems(uniqueItems);
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
    await acknowledge(item.id).catch(() => undefined);
  };

  const handleDownload = async (item: OutboxItem) => {
    const url = downloadUrl(item.id, token);
    setDownloadFallback(null);
    setDownloadingId(item.id);
    try {
      const opened = triggerDownload(url, item.displayName);
      if (!opened) {
        setDownloadFallback({ id: item.id, url });
        setError(t("downloadMayBeBlockedInThisBrowser"));
        return;
      }
      setError(t("downloadStarted"));
      await acknowledge(item.id).catch(() => undefined);
    } catch {
      setError(t("failedToDownloadFile"));
    } finally {
      setDownloadingId(null);
    }
  };

  const handleCopyDownloadLink = async (url: string) => {
    await writeClipboardText(new URL(url, window.location.href).toString());
    setError(t("downloadLinkCopied"));
  };

  const switchTab = (tab: "send" | "receive") => {
    activeTabRef.current = tab;
    setActiveTab(tab);
    if (tab === "receive") setHasNewDesktopItem(false);
  };

  return (
    <main className="mobile-upload-card">
      <h1>{t("appName")}</h1>
      <nav aria-label="Transfer direction">
        <button type="button" aria-selected={activeTab === "send"} onClick={() => switchTab("send")}>
          {t("sendToDesktop")}
        </button>
        <button type="button" aria-selected={activeTab === "receive"} onClick={() => switchTab("receive")}>
          {t("receive")} ({items.length})
        </button>
      </nav>
      {expired ? <p role="alert">{t("sessionExpired")}</p> : null}
      {error && !expired ? <p role="alert">{error}</p> : null}
      {hasNewDesktopItem && activeTab === "send" ? <p role="status">{t("newItemFromDesktop")}</p> : null}

      <section aria-label={t("sendToDesktop")} hidden={activeTab !== "send"}>
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

      <section aria-label={t("receiveFromDesktop")} hidden={activeTab !== "receive"}>
        <h2>{t("receiveFromDesktop")}</h2>
        {restrictedBrowser && hasDownloadableItems ? (
          <p role="note">{t("wechatDownloadHint")}</p>
        ) : null}
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
                    <>
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
                      <button type="button" onClick={() => void handleCopyDownloadLink(downloadUrl(item.id, token))}>
                        {t("copyDownloadLink")}
                      </button>
                    {downloadFallback?.id === item.id ? (
                      <div>
                        <p>{t("pleaseOpenSystemBrowserToDownload")}</p>
                        <a href={downloadFallback.url} target="_blank" rel="noreferrer">
                          {t("openInBrowserToDownload")}
                        </a>
                        <button type="button" onClick={() => void handleCopyDownloadLink(downloadFallback.url)}>
                          {t("copyDownloadLink")}
                        </button>
                      </div>
                    ) : null}
                    </>
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

function dedupeOutboxItems(items: OutboxItem[]): OutboxItem[] {
  const seen = new Set<string>();
  const unique: OutboxItem[] = [];

  for (const item of items) {
    if (seen.has(item.id)) continue;
    seen.add(item.id);
    unique.push(item);
  }

  return unique;
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

function triggerDownload(url: string, displayName?: string): boolean {
  if (isRestrictedDownloadBrowser()) return false;

  const link = document.createElement("a");
  link.href = url;
  link.download = displayName ?? "";
  link.target = "_blank";
  link.rel = "noopener";
  document.body.appendChild(link);
  link.click();
  link.remove();
  return true;
}

function isRestrictedDownloadBrowser(): boolean {
  return /MicroMessenger|WeChat|QQ\//i.test(navigator.userAgent);
}
