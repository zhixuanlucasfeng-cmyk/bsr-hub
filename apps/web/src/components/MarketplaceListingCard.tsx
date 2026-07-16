"use client";

import Link from "next/link";
import type { Listing } from "../lib/types";
import { listingPriceLabel, sellerFor } from "../lib/marketplace";
import { LinearIcon } from "./LinearIcon";
import { OptimizedImage } from "./OptimizedImage";

interface MarketplaceListingCardProps { listing: Listing; eager?: boolean }

export function MarketplaceListingCard({ listing, eager = false }: MarketplaceListingCardProps) {
  const seller = sellerFor(listing.ownerId);
  const distance = listing.distanceMiles ?? ((listing.id.length % 7) + 1) * 0.7;
  const typeLabel = listing.listingType === "workspace" ? "Space" : listing.listingType === "sale" ? "Second-hand" : "Rental";
  return <article className="group relative overflow-hidden rounded-card bg-white shadow-soft transition duration-200 hover:-translate-y-0.5 hover:shadow-[0_30px_75px_rgba(27,18,48,.16),0_5px_18px_rgba(27,18,48,.1)]">
    <Link href={`/listings/${listing.id}/`} className="block focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-brand" aria-label={`View ${listing.title}`}>
      <div className="relative aspect-[4/3] overflow-hidden bg-zinc-100"><OptimizedImage source={listing.imageSrc} alt={listing.imageAlt} eager={eager} className="absolute inset-0 size-full object-cover transition duration-300 group-hover:scale-[1.045]"/><span className="absolute left-3 top-3 rounded-full bg-white/92 px-3 py-1.5 text-[10px] font-extrabold uppercase tracking-[.1em] text-zinc-900 shadow-sm backdrop-blur">{typeLabel}</span><span className="absolute inset-x-3 bottom-3 translate-y-4 rounded-full bg-zinc-950/88 py-2.5 text-center text-xs font-bold text-white opacity-0 backdrop-blur transition group-hover:translate-y-0 group-hover:opacity-100">View & reserve</span></div>
      <div className="p-4.5">
        <p className="text-[10px] font-bold uppercase tracking-[.12em] text-brand">{listing.category} · {listing.condition}</p>
        <h3 className="mt-2 min-h-12 font-sans text-[17px] font-bold leading-6 text-zinc-950">{listing.title}</h3>
        <div className="mt-4 flex items-center gap-2.5"><span className="grid size-8 place-items-center rounded-full bg-zinc-950 text-[9px] font-extrabold text-white">{seller.initials}</span><span className="min-w-0 flex-1"><strong className="flex items-center gap-1 truncate text-xs text-zinc-800">{seller.name}<span className="grid size-3.5 place-items-center rounded-full bg-brand text-[8px] text-white">✓</span></strong><small className="text-[10px] text-zinc-500">{seller.rating.toFixed(1)} · {seller.reviewCount} reviews</small></span></div>
        <div className="mt-4 flex items-end justify-between border-t border-zinc-100 pt-4"><strong className="font-sans text-lg font-bold text-brand">{listingPriceLabel(listing)}</strong><span className="flex items-center gap-1 text-[10px] font-semibold text-zinc-500"><LinearIcon name="truck" className="size-3.5"/>{distance.toFixed(1)} mi</span></div>
      </div>
    </Link>
  </article>;
}
