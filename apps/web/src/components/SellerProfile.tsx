import type { Listing } from "../lib/types";
import { sellerFor } from "../lib/marketplace";
import { LinearIcon } from "./LinearIcon";

export function SellerProfile({ listing }: { listing: Listing }) {
  const seller = sellerFor(listing.ownerId);
  return <section className="flex flex-col gap-5 rounded-card bg-white p-5 shadow-sm sm:flex-row sm:items-center">
    <div className="grid size-14 shrink-0 place-items-center rounded-full border border-zinc-200 bg-zinc-950 text-sm font-extrabold text-white transition hover:scale-105">{seller.initials}</div>
    <div className="flex-1"><div className="flex flex-wrap items-center gap-2"><h2 className="font-[Manrope] text-lg font-bold text-zinc-950">{listing.sellerName ?? seller.name}</h2><span className="inline-flex items-center gap-1 rounded-full bg-violet-100 px-2.5 py-1 text-[10px] font-extrabold text-brand"><LinearIcon name="shield" className="size-3"/>Verified</span></div><p className="mt-1 text-xs text-zinc-500">{listing.rating ?? seller.rating} rating · {listing.reviewCount ?? seller.reviewCount} community reviews</p></div>
    <button className="rounded-full border border-zinc-200 px-5 py-2.5 text-xs font-bold text-zinc-800 transition hover:border-brand hover:bg-violet-50 hover:text-brand">Contact seller</button>
  </section>;
}
