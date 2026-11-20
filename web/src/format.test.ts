import { describe, expect, it } from "vitest";
import { direction, formatAsOf, formatChange, formatKwacha, formatPercent, formatPrice, timeAgo } from "./format";
import { withAlpha } from "./components/PriceChart";

describe("format", () => {
  it("formats prices with two decimals and separators", () => {
    expect(formatPrice(1774.99)).toBe("1,774.99");
    expect(formatPrice(17.4)).toBe("17.40");
  });

  it("signs changes with a real minus", () => {
    expect(formatChange(1.87)).toBe("+1.87");
    expect(formatChange(-0.78)).toBe("−0.78");
    expect(formatChange(0)).toBe("0.00");
    expect(formatPercent(0.9687)).toBe("+0.97%");
    expect(formatPercent(-0.1742)).toBe("−0.17%");
    expect(formatPercent(0)).toBe("0.00%");
    expect(formatPercent(-0.0006)).toBe("<0.01%");
    expect(formatPercent(0.004)).toBe("<0.01%");
  });

  it("formats kwacha compactly", () => {
    expect(formatKwacha(2_022_471_000)).toBe("MK 2.02B");
    expect(formatKwacha(640_060.54)).toBe("MK 640.06K");
  });

  it("classifies direction", () => {
    expect(direction(1)).toBe("up");
    expect(direction(-1)).toBe("down");
    expect(direction(0)).toBe("flat");
  });

  it("shows exchange time in Malawi time", () => {
    expect(formatAsOf("2026-11-20T13:00:00Z")).toBe("20 Nov 2026, 15:00");
  });

  it("describes age", () => {
    const now = new Date("2026-11-20T13:00:00Z");
    expect(timeAgo("2026-11-20T12:59:30Z", now)).toBe("just now");
    expect(timeAgo("2026-11-20T12:55:00Z", now)).toBe("5 min ago");
    expect(timeAgo("2026-11-20T10:00:00Z", now)).toBe("3 h ago");
    expect(timeAgo("2026-11-18T13:00:00Z", now)).toBe("2 d ago");
  });

  it("converts hex colours for the chart", () => {
    expect(withAlpha("#34d399", 0.3)).toBe("rgba(52, 211, 153, 0.3)");
    expect(withAlpha("not-a-colour", 0.3)).toBe("not-a-colour");
  });
});
