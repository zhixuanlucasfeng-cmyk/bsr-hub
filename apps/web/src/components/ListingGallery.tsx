"use client";

import { useRef, useState } from "react";
import type { Listing } from "../lib/types";
import { LinearIcon } from "./LinearIcon";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export function ListingGallery({ listing }: { listing: Listing }) {
  const images = [{ src:listing.imageSrc, alt:listing.imageAlt }, ...(listing.imageGallery ?? []).filter((image) => image.src !== listing.imageSrc)];
  const [active, setActive] = useState(0);
  const startX = useRef<number | null>(null);
  const previous = () => setActive((current) => (current - 1 + images.length) % images.length);
  const next = () => setActive((current) => (current + 1) % images.length);
  const finishSwipe = (x: number) => { if (startX.current === null) return; const distance = x - startX.current; startX.current = null; if (distance > 45) previous(); if (distance < -45) next(); };

  return <div>
    <div className="group relative aspect-[4/3] touch-pan-y overflow-hidden rounded-[24px] bg-zinc-100 shadow-soft" onPointerDown={(event) => { startX.current = event.clientX; }} onPointerUp={(event) => finishSwipe(event.clientX)} onPointerCancel={() => { startX.current = null; }}>
      {images.map((image, index) => <img key={image.src} src={`${basePath}${image.src}`} alt={image.alt} className={`absolute inset-0 size-full object-cover transition-opacity duration-300 ${index === active ? "opacity-100" : "pointer-events-none opacity-0"}`}/>)}
      {images.length > 1 && <><button onClick={previous} aria-label="Previous image" className="absolute left-4 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-white/90 text-zinc-900 shadow-md backdrop-blur transition hover:scale-105"><LinearIcon name="chevron" className="size-5 rotate-180"/></button><button onClick={next} aria-label="Next image" className="absolute right-4 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-white/90 text-zinc-900 shadow-md backdrop-blur transition hover:scale-105"><LinearIcon name="chevron" className="size-5"/></button></>}
      <span className="absolute bottom-4 right-4 rounded-full bg-zinc-950/75 px-3 py-1.5 text-[10px] font-bold text-white backdrop-blur">{active + 1} / {images.length}</span>
    </div>
    {images.length > 1 && <div className="mt-4 flex gap-3 overflow-x-auto pb-2">{images.map((image, index) => <button key={image.src} onClick={() => setActive(index)} aria-label={`View image ${index + 1}`} className={`relative aspect-square w-20 shrink-0 overflow-hidden rounded-xl transition ${index === active ? "ring-3 ring-brand ring-offset-2" : "opacity-65 hover:opacity-100"}`}><img src={`${basePath}${image.src}`} alt="" className="size-full object-cover"/></button>)}</div>}
  </div>;
}
