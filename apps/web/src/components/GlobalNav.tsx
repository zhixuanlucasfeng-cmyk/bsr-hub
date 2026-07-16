"use client";

import { useEffect, useState } from "react";
import { BrandLogo } from "./BrandLogo";
import { LinearIcon } from "./LinearIcon";

interface GlobalNavProps {
  activeView: "market" | "orders" | "create";
  orderCount: number;
  initials: string;
  onExplore: () => void;
  onCategories: () => void;
  onOrders: () => void;
  onList: () => void;
  onAvatar: () => void;
}

export function GlobalNav({ activeView, orderCount, initials, onExplore, onCategories, onOrders, onList, onAvatar }: GlobalNavProps) {
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => {
    const update = () => setScrolled(window.scrollY > 18);
    update();
    window.addEventListener("scroll", update, { passive: true });
    return () => window.removeEventListener("scroll", update);
  }, []);

  return <header className={`fixed inset-x-0 top-0 z-40 h-[76px] transition-all duration-200 ${scrolled ? "border-b border-black/5 bg-white/78 shadow-[0_10px_35px_rgba(24,16,45,.08)] backdrop-blur-xl" : "bg-[#fafafa]/96"}`}>
    <div className="mx-auto flex h-full max-w-[1440px] items-center gap-8 px-5 sm:px-8 lg:px-12">
      <button onClick={onExplore} className="shrink-0 rounded-xl focus-visible:outline-3 focus-visible:outline-offset-4 focus-visible:outline-brand" aria-label="BSR Hub home"><BrandLogo variant="horizontal" className="h-10 w-auto sm:h-11"/></button>
      <nav className="global-nav-links hidden flex-1 items-center justify-center gap-1 md:flex" aria-label="Primary navigation">
        <button onClick={onExplore} className={`rounded-full px-5 py-2.5 text-sm font-semibold transition ${activeView === "market" ? "bg-violet-100 text-brand" : "text-zinc-600 hover:bg-white hover:text-zinc-950"}`}>Explore</button>
        <button onClick={onCategories} className="rounded-full px-5 py-2.5 text-sm font-semibold text-zinc-600 transition hover:bg-white hover:text-zinc-950">Categories</button>
        <button onClick={onOrders} className={`flex items-center gap-2 rounded-full px-5 py-2.5 text-sm font-semibold transition ${activeView === "orders" ? "bg-violet-100 text-brand" : "text-zinc-600 hover:bg-white hover:text-zinc-950"}`}>My orders <span className="rounded-full bg-accent px-2 py-0.5 text-[10px] text-zinc-950">{orderCount}</span></button>
      </nav>
      <div className="ml-auto flex items-center gap-2 sm:gap-3">
        <button onClick={onOrders} className="grid size-10 place-items-center rounded-full text-zinc-700 transition hover:bg-white md:hidden" aria-label="My orders"><LinearIcon name="orders" className="size-5"/></button>
        <button onClick={onList} className="group relative hidden overflow-hidden rounded-full bg-gradient-to-r from-brand to-violet-500 px-5 py-3 text-sm font-bold text-white shadow-lg shadow-violet-500/20 transition hover:-translate-y-0.5 hover:from-brand-deep hover:to-brand sm:block"><span className="absolute inset-y-0 -left-1/2 w-1/3 skew-x-[-20deg] bg-white/25 transition-all duration-500 group-hover:left-[120%]"/>+ List something</button>
        <button onClick={onAvatar} className="grid size-11 place-items-center rounded-full bg-zinc-950 text-xs font-extrabold text-white ring-4 ring-white transition hover:scale-105" aria-label="Open demo account">{initials}</button>
      </div>
    </div>
  </header>;
}
