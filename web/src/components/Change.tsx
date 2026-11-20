import { direction, formatChange, formatPercent } from "../format";

/** Coloured percentage pill with an arrow, e.g. ▲ +0.97%. */
export function ChangePill({ pct, size = "md" }: { pct: number; size?: "md" | "lg" }) {
  const dir = direction(pct);
  const arrow = dir === "up" ? "▲" : dir === "down" ? "▼" : "•";
  return (
    <span className={`pill pill-${dir} pill-${size}`}>
      <span aria-hidden="true">{arrow}</span> {formatPercent(pct)}
    </span>
  );
}

/** Plain coloured absolute change, e.g. +1.87. */
export function ChangeText({ value }: { value: number }) {
  return <span className={`text-${direction(value)}`}>{formatChange(value)}</span>;
}
