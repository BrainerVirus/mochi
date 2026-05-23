import { useEffect } from "react";

/** Sync shadcn `.dark` on `<html>` with the OS light/dark preference. */
export function useSystemColorScheme(): void {
  useEffect(() => {
    if (typeof window === "undefined") {
      return undefined;
    }

    const media = window.matchMedia("(prefers-color-scheme: dark)");

    const apply = () => {
      document.documentElement.classList.toggle("dark", media.matches);
    };

    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, []);
}
