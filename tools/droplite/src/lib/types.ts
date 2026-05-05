export type ReceivedKind = "text" | "file" | "image";

export interface ReceivedItem {
  id: string;
  kind: ReceivedKind;
  name: string;
  text?: string;
  path?: string;
  size?: number;
  mime?: string;
  received_at: number;
}

export interface SessionView {
  connection_url: string;
  device_name: string;
  expires_at: number;
  local_ip: string;
  max_upload_bytes: number;
  port: number;
  receive_dir: string;
  security_note: string;
  started_at: number;
}

export interface DesktopState {
  session: SessionView;
  received: ReceivedItem[];
}
