import type { Listing } from "../lib/types";
import { relatedListings } from "../lib/marketplace";
import { MarketplaceListingCard } from "./MarketplaceListingCard";

export function RelatedListings({ listing, listings }: { listing: Listing; listings: Listing[] }) {
  const related = relatedListings(listing, listings, 6);
  return <section className="mt-24"><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Keep exploring</p><h2 className="mt-2 font-sans text-4xl font-bold tracking-tight text-zinc-950">Similar listings nearby</h2><div className="mt-8 grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">{related.slice(0, 4).map((item) => <MarketplaceListingCard key={item.id} listing={item}/>)}</div></section>;
}
