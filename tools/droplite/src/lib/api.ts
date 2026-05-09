import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { DesktopState, OutboxItem, ReceivedItem } from "./types";

export function getDesktopState(): Promise<DesktopState> {
  return invoke<DesktopState>("get_desktop_state");
}

export function refreshSession(): Promise<DesktopState> {
  return invoke<DesktopState>("refresh_session");
}

export function openReceiveFolder(): Promise<void> {
  return invoke<void>("open_receive_folder");
}

export function getReceiveDirectory(): Promise<string> {
  return invoke<string>("get_receive_directory");
}

export function setReceiveDirectory(path: string): Promise<string> {
  return invoke<string>("set_receive_directory", { path });
}

export function resetReceiveDirectory(): Promise<string> {
  return invoke<string>("reset_receive_directory");
}

export async function chooseReceiveDirectory(title: string): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title
  });

  return Array.isArray(selected) ? (selected[0] ?? null) : selected;
}

export function addOutboxText(content: string): Promise<OutboxItem> {
  return invoke<OutboxItem>("add_outbox_text", { content });
}

export function addOutboxFile(path: string): Promise<OutboxItem> {
  return invoke<OutboxItem>("add_outbox_file", { path });
}

export function listOutboxItems(): Promise<OutboxItem[]> {
  return invoke<OutboxItem[]>("list_outbox_items");
}

export async function chooseOutboxFiles(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    directory: false,
    title: "Choose files"
  });

  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}

export function onReceivedItem(handler: (item: ReceivedItem) => void): Promise<() => void> {
  return listen<ReceivedItem>("droplite://received", (event) => handler(event.payload));
}
