"use client";

import { LinearIcon, type IconName } from "./LinearIcon";
import { OptimizedImage } from "./OptimizedImage";

interface BusinessShowcaseProps { onRent: () => void; onWorkspace: () => void; onSecondHand: () => void }
const cards: Array<{ title: string; copy: string; image: string; icon: IconName; action: keyof BusinessShowcaseProps }> = [
  { title: "Rent products", copy: "Consoles, cameras, computers, and tools — without the ownership cost.", image: "/images/categories/gaming.jpg", icon: "game", action: "onRent" },
  { title: "Book creative spaces", copy: "Studios, workshops, print rooms, and supervised maker equipment.", image: "/images/categories/studios.jpg", icon: "studio", action: "onWorkspace" },
  { title: "Buy second-hand", copy: "Useful technology at a fair price with protected BSR payment.", image: "/images/categories/second-hand.jpg", icon: "reuse", action: "onSecondHand" },
];

export function BusinessShowcase(props: BusinessShowcaseProps) {
  return <section className="px-5 pb-24 sm:px-8 lg:px-12"><div className="mx-auto max-w-[1320px]"><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Three ways to share smarter</p><h2 className="mt-2 font-sans text-4xl font-bold tracking-tight text-zinc-950">Access what you need</h2><div className="mt-8 grid gap-5 lg:grid-cols-3">{cards.map((card) => <article key={card.title} className="group relative min-h-[390px] overflow-hidden rounded-[24px] shadow-soft"><OptimizedImage source={card.image} alt="" sizes="(max-width: 1024px) 100vw, 33vw" className="absolute inset-0 size-full object-cover transition duration-500 group-hover:scale-105"/><div className="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/48 to-transparent"/><div className="absolute inset-x-0 bottom-0 p-7 text-white"><span className="mb-5 grid size-11 place-items-center rounded-xl bg-white/14 backdrop-blur"><LinearIcon name={card.icon} className="size-5"/></span><h3 className="font-sans text-2xl font-bold">{card.title}</h3><p className="mt-3 text-sm leading-6 text-white/74">{card.copy}</p><button onClick={props[card.action]} className="mt-6 rounded-full border border-white/35 bg-white/10 px-5 py-2.5 text-xs font-bold backdrop-blur transition hover:bg-white hover:text-zinc-950">Explore now →</button></div></article>)}</div></div></section>;
}
