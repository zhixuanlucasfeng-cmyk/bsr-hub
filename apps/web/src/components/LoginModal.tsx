"use client";

import { useEffect } from "react";
import { LinearIcon } from "./LinearIcon";

interface DemoPersona { id: string; name: string; role: string; initials: string }
interface LoginModalProps { open: boolean; personas: readonly DemoPersona[]; onSelect: (id: string) => void; onGuest: () => void; onClose: () => void }

export function LoginModal({ open, personas, onSelect, onGuest, onClose }: LoginModalProps) {
  useEffect(() => {
    if (!open) return;
    const close = (event: KeyboardEvent) => { if (event.key === "Escape") onClose(); };
    document.addEventListener("keydown", close);
    return () => document.removeEventListener("keydown", close);
  }, [open, onClose]);
  if (!open) return null;

  return <div className="fixed inset-0 z-[80] grid place-items-center bg-zinc-950/55 p-4 backdrop-blur-sm" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <section role="dialog" aria-modal="true" aria-labelledby="login-title" className="relative w-full max-w-lg overflow-hidden rounded-[28px] bg-white p-7 shadow-[0_35px_100px_rgba(0,0,0,.3)] sm:p-9">
      <button onClick={onClose} className="absolute right-5 top-5 grid size-9 place-items-center rounded-full bg-zinc-100 text-xl text-zinc-600 transition hover:bg-zinc-200" aria-label="Close sign in">×</button>
      <div className="mb-6 grid size-12 place-items-center rounded-2xl bg-accent text-zinc-950"><LinearIcon name="user" className="size-6"/></div>
      <p className="mb-2 text-xs font-extrabold uppercase tracking-[.18em] text-brand">Demo sign in</p>
      <h2 id="login-title" className="font-[Manrope] text-3xl font-bold tracking-tight text-zinc-950">Welcome to BSR Hub</h2>
      <p className="mt-3 text-sm leading-6 text-zinc-600">Choose a fictional identity to try protected orders and listing tools. No real credentials or identity documents are collected.</p>
      <div className="mt-7 space-y-3" aria-label="Demo identities">
        {personas.map((persona) => <button key={persona.id} onClick={() => onSelect(persona.id)} className="flex w-full items-center gap-4 rounded-2xl bg-zinc-50 p-4 text-left transition hover:-translate-y-0.5 hover:bg-violet-50 hover:shadow-md">
          <span className="grid size-11 place-items-center rounded-full bg-zinc-950 text-xs font-extrabold text-white">{persona.initials}</span>
          <span className="flex-1"><strong className="block text-sm text-zinc-950">{persona.name}</strong><small className="text-zinc-500">{persona.role} · Fictional demo account</small></span>
          <LinearIcon name="chevron" className="size-5 text-brand"/>
        </button>)}
      </div>
      <button onClick={onGuest} className="mt-5 w-full rounded-full border border-zinc-200 px-5 py-3 text-sm font-semibold text-zinc-700 transition hover:bg-zinc-50">Browse as guest</button>
      <p className="mt-4 text-center text-[11px] leading-5 text-zinc-400">Protected actions will ask you to select a demo identity.</p>
    </section>
  </div>;
}
