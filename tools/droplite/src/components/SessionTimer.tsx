import { useEffect, useMemo, useState } from "react";
import { Clock } from "lucide-react";
import { formatClock } from "../lib/format";
import { useI18n } from "../i18n";

interface SessionTimerProps {
  expiresAt: number;
}

export function SessionTimer({ expiresAt }: SessionTimerProps) {
  const { t } = useI18n();
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Math.floor(Date.now() / 1000)), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const secondsLeft = useMemo(() => Math.max(0, expiresAt - now), [expiresAt, now]);

  return (
    <div className={`timer ${secondsLeft === 0 ? "expired" : ""}`}>
      <Clock size={18} />
      <span>
        {secondsLeft === 0
          ? t("sessionExpired")
          : t("sessionExpiresIn", { time: formatClock(secondsLeft) })}
      </span>
    </div>
  );
}
