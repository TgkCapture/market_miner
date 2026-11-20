const price = new Intl.NumberFormat("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const whole = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });
const compact = new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 2 });

export const formatPrice = (n: number) => price.format(n);

export const formatNumber = (n: number) => whole.format(n);

export const formatCompact = (n: number) => compact.format(n);

/** Malawi kwacha, compact: "MK 2.02B". */
export const formatKwacha = (n: number) => `MK ${compact.format(n)}`;

/** Signed change: "+1.87", "−0.78", "0.00". Uses a real minus sign. */
export function formatChange(n: number): string {
  if (n === 0) return price.format(0);
  return `${n > 0 ? "+" : "−"}${price.format(Math.abs(n))}`;
}

/** Signed percentage: "+0.97%", "−0.17%", "0.00%". Tiny non-zero moves read "<0.01%". */
export function formatPercent(n: number): string {
  if (n === 0) return "0.00%";
  if (Math.abs(n) < 0.005) return "<0.01%";
  return `${n > 0 ? "+" : "−"}${price.format(Math.abs(n))}%`;
}

export type Direction = "up" | "down" | "flat";

export const direction = (n: number): Direction => (n > 0 ? "up" : n < 0 ? "down" : "flat");

/** "20 Nov 2026, 15:00" in Malawi time, where the exchange is. */
export function formatAsOf(iso: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "Africa/Blantyre",
  }).format(new Date(iso));
}

/** "just now", "5 min ago", "3 h ago", "2 d ago". */
export function timeAgo(iso: string, now: Date = new Date()): string {
  const secs = Math.max(0, Math.round((now.getTime() - new Date(iso).getTime()) / 1000));
  if (secs < 60) return "just now";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins} min ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours} h ago`;
  return `${Math.floor(hours / 24)} d ago`;
}
