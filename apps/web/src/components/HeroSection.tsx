"use client";

import { LinearIcon } from "./LinearIcon";
import { OptimizedImage } from "./OptimizedImage";

interface HeroSectionProps { onExplore: () => void; onList: () => void }

const scenes = [
  { src: "/images/listings/ps5-slim.jpg", alt: "PlayStation console rental", label: "Console rental", price: "from $12", className: "left-0 top-6 w-[58%] -rotate-2" },
  { src: "/images/listings/tool-set.jpg", alt: "Power tool rental", label: "Tools by the day", price: "from $28", className: "right-0 top-0 w-[47%] rotate-3" },
  { src: "/images/listings/photo-studio.jpg", alt: "Creative studio booking", label: "Creative studios", price: "from $35", className: "bottom-1 right-[8%] w-[64%] -rotate-1" },
];

export function HeroSection({ onExplore, onList }: HeroSectionProps) {
  return <section className="relative isolate overflow-hidden px-5 pb-20 pt-32 sm:px-8 lg:px-12 lg:pb-28 lg:pt-40">
    <div className="absolute inset-0 -z-10 bg-[radial-gradient(circle_at_1px_1px,rgba(124,58,237,.10)_1px,transparent_0)] [background-size:24px_24px] [mask-image:linear-gradient(to_bottom,black,transparent_82%)]"/>
    <div className="absolute -left-32 top-16 -z-10 size-[420px] rounded-full bg-violet-200/45 blur-3xl"/>
    <div className="mx-auto grid max-w-[1320px] items-center gap-16 lg:grid-cols-[1.02fr_.98fr]">
      <div>
        <p className="mb-5 text-xs font-extrabold uppercase tracking-[.2em] text-brand">Community access, without the price tag</p>
        <h1 className="max-w-3xl font-sans text-6xl font-bold leading-[.94] tracking-[-.065em] text-zinc-950 sm:text-7xl lg:text-[92px]">Use more.<br/><span className="bg-gradient-to-r from-brand to-violet-500 bg-clip-text text-transparent">Own less.</span></h1>
        <p className="mt-7 max-w-xl text-lg leading-8 text-zinc-600">Rent products, book creative spaces, and give second-hand goods a new life — from trusted people nearby.</p>
        <div className="mt-8 flex flex-col gap-3 sm:flex-row"><button onClick={onExplore} className="rounded-full bg-gradient-to-r from-brand to-violet-500 px-7 py-4 text-sm font-bold text-white shadow-lg shadow-violet-500/20 transition hover:-translate-y-0.5 hover:from-brand-deep hover:to-brand">Explore nearby</button><button onClick={onList} className="rounded-full border border-zinc-200 bg-white px-7 py-4 text-sm font-bold text-zinc-900 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md">Earn from your things →</button></div>
        <div className="mt-8 flex flex-wrap gap-x-6 gap-y-3 text-xs font-semibold text-zinc-700">
          <span className="flex items-center gap-2"><LinearIcon name="shield" className="size-4 text-brand"/>Protected payments</span>
          <span className="flex items-center gap-2"><LinearIcon name="user" className="size-4 text-brand"/>Verified community</span>
          <span className="flex items-center gap-2"><LinearIcon name="truck" className="size-4 text-brand"/>Local delivery</span>
        </div>
      </div>
      <div className="relative mx-auto h-[430px] w-full max-w-[610px] sm:h-[520px]">
        <div className="absolute inset-8 rounded-[42px] bg-zinc-950 shadow-[0_45px_100px_rgba(30,18,55,.2)]"/>
        {scenes.map((scene, index) => <article key={scene.label} className={`group absolute overflow-hidden rounded-[24px] bg-white p-2 shadow-[0_25px_70px_rgba(30,18,55,.24),0_4px_15px_rgba(30,18,55,.12)] transition duration-200 hover:z-10 hover:-translate-y-2 hover:rotate-0 ${scene.className}`}>
          <div className="relative aspect-[4/3] overflow-hidden rounded-[18px]"><OptimizedImage source={scene.src} alt={scene.alt} eager={index === 0} sizes="(max-width: 1024px) 55vw, 32vw" className="absolute inset-0 size-full object-cover transition duration-300 group-hover:scale-105"/><div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/75 to-transparent p-4 pt-12 text-white"><strong className="block text-sm">{scene.label}</strong><span className="mt-1 inline-block rounded-full bg-accent px-2.5 py-1 text-[10px] font-extrabold text-zinc-950">{scene.price}</span></div></div>
        </article>)}
      </div>
    </div>
  </section>;
}
