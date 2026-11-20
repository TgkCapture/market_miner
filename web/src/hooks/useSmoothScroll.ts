import { useEffect } from "react";
import { useLocation } from "react-router-dom";
import Lenis from "lenis";

let lenis: Lenis | null = null;

/** Inertial smooth scrolling for the whole page. Turned off for people who prefer reduced motion. */
export function useSmoothScroll() {
  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    lenis = new Lenis({ duration: 1.1, smoothWheel: true, wheelMultiplier: 0.9 });
    let frame = 0;
    const raf = (time: number) => {
      lenis?.raf(time);
      frame = requestAnimationFrame(raf);
    };
    frame = requestAnimationFrame(raf);

    return () => {
      cancelAnimationFrame(frame);
      lenis?.destroy();
      lenis = null;
    };
  }, []);
}

/** Scrolls to the top on every route change. */
export function useScrollToTopOnNavigate() {
  const { pathname } = useLocation();
  useEffect(() => {
    if (lenis) lenis.scrollTo(0, { immediate: true });
    else window.scrollTo(0, 0);
  }, [pathname]);
}

/** Smoothly scrolls to an element, using Lenis when it is running. */
export function scrollToElement(el: HTMLElement) {
  if (lenis) lenis.scrollTo(el, { offset: -80 });
  else el.scrollIntoView({ behavior: "smooth", block: "start" });
}
