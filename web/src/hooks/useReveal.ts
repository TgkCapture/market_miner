import { useEffect } from "react";

/**
 * Fades `.reveal` elements in as they scroll into view. Re-scans when `key` changes so
 * elements rendered after data loads are picked up too.
 */
export function useReveal(key: unknown) {
  useEffect(() => {
    if (typeof IntersectionObserver === "undefined") return;
    document.documentElement.classList.add("reveal-ready");

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            entry.target.classList.add("is-visible");
            observer.unobserve(entry.target);
          }
        }
      },
      { rootMargin: "0px 0px -40px 0px", threshold: 0.05 },
    );
    document.querySelectorAll(".reveal:not(.is-visible)").forEach((el, i) => {
      (el as HTMLElement).style.setProperty("--reveal-delay", `${Math.min(i, 6) * 60}ms`);
      observer.observe(el);
    });
    return () => observer.disconnect();
  }, [key]);
}
