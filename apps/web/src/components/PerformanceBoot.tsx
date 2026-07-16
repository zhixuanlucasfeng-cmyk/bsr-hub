"use client";

import { useEffect } from "react";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";
const rootPath = basePath.replace(/\/(?:hub|runner)\/?$/, "");

export function PerformanceBoot() {
  useEffect(() => {
    if (process.env.NODE_ENV !== "production" || !("serviceWorker" in navigator)) return;
    const register = () => {
      void navigator.serviceWorker.register(`${rootPath}/sw.js`, {
        scope: `${rootPath || ""}/`,
      }).catch(() => undefined);
    };
    window.addEventListener("load", register, { once: true });
    return () => window.removeEventListener("load", register);
  }, []);
  return null;
}
