import { Upload } from "lucide-react";

interface UploadPanelProps {
  status: string;
}

export function UploadPanel({ status }: UploadPanelProps) {
  return (
    <section className="mobile-upload-card">
      <Upload size={28} />
      <h1>Drop to this computer</h1>
      <p>{status}</p>
    </section>
  );
}
