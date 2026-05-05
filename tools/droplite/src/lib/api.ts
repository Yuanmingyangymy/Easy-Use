import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DesktopState, ReceivedItem } from "./types";

export function getDesktopState(): Promise<DesktopState> {
  return invoke<DesktopState>("get_desktop_state");
}

export function refreshSession(): Promise<DesktopState> {
  return invoke<DesktopState>("refresh_session");
}

export function openReceiveFolder(): Promise<void> {
  return invoke<void>("open_receive_folder");
}

export function onReceivedItem(handler: (item: ReceivedItem) => void): Promise<() => void> {
  return listen<ReceivedItem>("droplite://received", (event) => handler(event.payload));
}
