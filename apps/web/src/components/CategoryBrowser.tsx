"use client";

import type { IconName } from "./LinearIcon";
import { LinearIcon } from "./LinearIcon";
import type { CategoryId } from "../lib/categories";

interface CategoryBrowserProps { selected: CategoryId; onSelect: (categoryId: CategoryId) => void }

const categoryItems: Array<{ id: CategoryId; label: string; icon: IconName }> = [
  { id: "gaming", label: "Gaming consoles", icon: "game" },
  { id: "cameras", label: "Camera gear", icon: "camera" },
  { id: "tools", label: "Power tools", icon: "tool" },
  { id: "computers", label: "Office equipment", icon: "laptop" },
  { id: "studios", label: "Creative spaces", icon: "studio" },
  { id: "second-hand", label: "Second-hand tech", icon: "reuse" },
];

export function CategoryBrowser({ selected, onSelect }: CategoryBrowserProps) {
  return <section id="categories" className="scroll-mt-28 px-5 py-6 sm:px-8 lg:px-12" aria-labelledby="category-heading">
    <div className="mx-auto max-w-[1320px]">
      <div className="mb-5 flex items-end justify-between"><div><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Browse the community</p><h2 id="category-heading" className="mt-2 font-sans text-3xl font-bold tracking-tight text-zinc-950">Find your category</h2></div>{selected !== "all" && <button className="text-sm font-bold text-brand hover:text-brand-deep" onClick={() => onSelect("all")}>View all</button>}</div>
      <div className="flex gap-3 overflow-x-auto pb-4 [scrollbar-width:none]">
        {categoryItems.map((category) => { const active = selected === category.id; return <button key={category.id} aria-pressed={active} onClick={() => onSelect(active ? "all" : category.id)} className={`flex min-w-[174px] items-center gap-3 rounded-card px-4 py-4 text-left shadow-sm transition hover:-translate-y-0.5 hover:shadow-md ${active ? "bg-brand text-white" : "bg-white text-zinc-800"}`}><span className={`grid size-11 shrink-0 place-items-center rounded-xl ${active ? "bg-white/16" : "bg-violet-50 text-brand"}`}><LinearIcon name={category.icon} className="size-5"/></span><span className="text-sm font-bold leading-5">{category.label}</span></button>; })}
      </div>
    </div>
  </section>;
}
