"use client";

import { useEffect, useState } from "react";
import { useAuth } from "./AuthProvider";
import { LinearIcon } from "./LinearIcon";

interface AuthModalProps {
  open: boolean;
  onClose: () => void;
  onUseDemo: () => void;
}

export function AuthModal({ open, onClose, onUseDemo }: AuthModalProps) {
  const { configured, user, profile, signIn, signOut, signUp } = useAuth();
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", close);
    return () => document.removeEventListener("keydown", close);
  }, [open, onClose]);

  if (!open) return null;

  async function submit() {
    setError("");
    setBusy(true);
    try {
      if (mode === "signin") await signIn(email, password);
      else await signUp(email, password);
      onClose();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Account action failed.");
    } finally {
      setBusy(false);
    }
  }

  async function logout() {
    setBusy(true);
    await signOut();
    setBusy(false);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-[90] grid place-items-center bg-zinc-950/55 p-4 backdrop-blur-sm"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="real-auth-title"
        className="relative w-full max-w-md overflow-hidden rounded-[28px] bg-white p-7 shadow-[0_35px_100px_rgba(0,0,0,.3)] sm:p-8"
      >
        <button
          onClick={onClose}
          className="absolute right-5 top-5 grid size-9 place-items-center rounded-full bg-zinc-100 text-xl text-zinc-600 transition hover:bg-zinc-200"
          aria-label="Close account dialog"
        >
          x
        </button>
        <div className="mb-6 grid size-12 place-items-center rounded-2xl bg-accent text-zinc-950">
          <LinearIcon name="user" className="size-6" />
        </div>
        <p className="mb-2 text-xs font-extrabold uppercase tracking-[.18em] text-brand">
          BSR Hub account
        </p>
        <h2 id="real-auth-title" className="font-sans text-3xl font-bold tracking-tight text-zinc-950">
          {user ? "Your account" : mode === "signin" ? "Sign in" : "Create account"}
        </h2>
        <p className="mt-3 text-sm leading-6 text-zinc-600">
          Real account demo uses Supabase Auth and the Rust API. Do not enter real payment, identity, or private address data.
        </p>

        {!configured ? (
          <div className="mt-6 space-y-4">
            <p className="rounded-2xl bg-violet-50 p-4 text-sm leading-6 text-violet-900">
              Online accounts are not connected on this build yet. You can still use the fictional demo identity flow.
            </p>
            <button
              onClick={onUseDemo}
              className="w-full rounded-full bg-brand px-5 py-3 text-sm font-bold text-white transition hover:bg-brand-deep"
            >
              Continue with demo identity
            </button>
          </div>
        ) : user ? (
          <div className="mt-6 space-y-4">
            <div className="rounded-2xl bg-zinc-50 p-4">
              <p className="text-sm font-bold text-zinc-950">
                {profile?.displayName ?? user.email ?? "BSR member"}
              </p>
              <p className="mt-1 text-xs text-zinc-500">{user.email}</p>
            </div>
            <button
              disabled={busy}
              onClick={logout}
              className="w-full rounded-full border border-zinc-200 px-5 py-3 text-sm font-bold text-zinc-700 transition hover:bg-zinc-50 disabled:opacity-60"
            >
              Sign out
            </button>
          </div>
        ) : (
          <div className="mt-6 space-y-4">
            <input
              className="w-full rounded-xl border border-zinc-200 px-4 py-3 text-sm outline-none transition focus:border-brand focus:ring-4 focus:ring-violet-100"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="Email"
            />
            <input
              className="w-full rounded-xl border border-zinc-200 px-4 py-3 text-sm outline-none transition focus:border-brand focus:ring-4 focus:ring-violet-100"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Password"
            />
            {error ? <p className="text-sm text-red-600">{error}</p> : null}
            <button
              disabled={busy}
              onClick={submit}
              className="w-full rounded-full bg-gradient-to-r from-brand to-violet-500 px-5 py-3 text-sm font-bold text-white transition hover:-translate-y-0.5 hover:from-brand-deep hover:to-brand disabled:opacity-60"
            >
              {busy ? "Working..." : mode === "signin" ? "Sign in" : "Create account"}
            </button>
            <button
              onClick={() => setMode(mode === "signin" ? "signup" : "signin")}
              className="w-full text-sm font-bold text-brand"
            >
              {mode === "signin" ? "Need an account? Create one" : "Already have an account? Sign in"}
            </button>
            <button onClick={onUseDemo} className="w-full text-xs font-semibold text-zinc-500">
              Use fictional demo identity instead
            </button>
          </div>
        )}
      </section>
    </div>
  );
}
