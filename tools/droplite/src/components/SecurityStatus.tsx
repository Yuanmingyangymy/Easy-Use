import { ShieldCheck } from "lucide-react";
import type { SessionView } from "../lib/types";

interface SecurityStatusProps {
  session: SessionView;
}

export function SecurityStatus({ session }: SecurityStatusProps) {
  return (
    <section className="security-panel" aria-label="Security status">
      <ShieldCheck size={20} />
      <div>
        <strong>Local network only</strong>
        <p>{session.security_note}</p>
      </div>
    </section>
  );
}
