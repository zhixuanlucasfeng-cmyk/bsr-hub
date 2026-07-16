"use client";

import { useEffect } from "react";

interface IntentRedirectProps {
  intent: "orders" | "create";
}

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export function IntentRedirect({ intent }: IntentRedirectProps) {
  const destination = `${basePath}/?intent=${intent}`;

  useEffect(() => {
    window.location.replace(destination);
  }, [destination]);

  return <main className="grid min-h-screen place-items-center bg-canvas px-5 text-center text-ink"><div className="max-w-md rounded-[24px] bg-white p-9 shadow-soft"><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">BSR Hub</p><h1 className="mt-3 font-sans text-3xl font-bold text-zinc-950">Opening {intent === "orders" ? "your orders" : "the listing form"}…</h1><p className="mt-4 text-sm leading-6 text-zinc-500">This direct link uses the same protected demo-session flow as the marketplace.</p><a href={destination} className="mt-7 inline-flex rounded-full bg-brand px-6 py-3 text-xs font-bold text-white transition hover:bg-brand-deep">Continue to BSR Hub</a></div></main>;
}
