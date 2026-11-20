import { useCallback, useEffect, useState } from "react";

export type Theme = "light" | "dark";

function systemTheme(): Theme {
  return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function savedTheme(): Theme | null {
  try {
    const t = localStorage.getItem("mm-theme");
    return t === "light" || t === "dark" ? t : null;
  } catch {
    return null;
  }
}

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => savedTheme() ?? systemTheme());

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const toggle = useCallback(() => {
    setTheme((t) => {
      const next = t === "dark" ? "light" : "dark";
      try {
        localStorage.setItem("mm-theme", next);
      } catch {
        /* storage unavailable; the choice just won't persist */
      }
      return next;
    });
  }, []);

  return { theme, toggle };
}
