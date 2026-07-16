"use client";

import type { Listing } from "../lib/types";
import { MarketplaceListingCard } from "./MarketplaceListingCard";
import { LinearIcon } from "./LinearIcon";
import { RevealOnScroll } from "./RevealOnScroll";

interface FeaturedListingsProps {
  listings: Listing[];
  query: string;
  listingType: string;
  onQuery: (query: string) => void;
  onType: (type: string) => void;
}

export function FeaturedListings({ listings, query, listingType, onQuery, onType }: FeaturedListingsProps) {
  return <section id="market" className="scroll-mt-24 px-5 py-20 sm:px-8 lg:px-12">
    <div className="mx-auto max-w-[1320px]">
      <header className="mb-9 flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between"><div><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Near Babson</p><h2 className="mt-2 font-[Manrope] text-4xl font-bold tracking-[-.035em] text-zinc-950 sm:text-5xl">Popular rentals & finds</h2><p className="mt-3 text-sm text-zinc-500">Real examples from fictional demo sellers — no real payment is collected.</p></div><div className="flex flex-col gap-3 sm:flex-row"><label className="flex min-w-[280px] items-center gap-3 rounded-full bg-white px-4 py-3 shadow-sm ring-1 ring-zinc-100"><LinearIcon name="search" className="size-5 text-zinc-400"/><span className="sr-only">Search listings</span><input value={query} onChange={(event) => onQuery(event.target.value)} placeholder="Search PS5, studio, camera…" className="w-full bg-transparent text-sm outline-none placeholder:text-zinc-400"/></label><select aria-label="Listing type" value={listingType} onChange={(event) => onType(event.target.value)} className="rounded-full bg-white px-5 py-3 text-sm font-semibold text-zinc-700 shadow-sm ring-1 ring-zinc-100 outline-none"><option value="all">All types</option><option value="rental">Rentals</option><option value="workspace">Workspaces</option><option value="sale">Second-hand</option></select></div></header>
      {listings.length ? <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 xl:grid-cols-4">{listings.map((listing, index) => <RevealOnScroll key={listing.id} delay={(index % 4) * 70}><MarketplaceListingCard listing={listing} eager={index < 4}/></RevealOnScroll>)}</div> : <div className="rounded-card bg-white p-16 text-center text-zinc-500 shadow-soft">No listings match these filters.</div>}
    </div>
  </section>;
}
