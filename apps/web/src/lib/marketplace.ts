import type { Fulfillment, Listing } from "./types.ts";

export const demoSessionKey = "bsr-demo-session";

const sellers: Record<string, { name: string; initials: string; verified: boolean; rating: number; reviewCount: number }> = {
  "seller-demo": { name: "Jordan Lee", initials: "JL", verified: true, rating: 4.9, reviewCount: 87 },
  "tech-demo": { name: "Northstar Tech", initials: "NT", verified: true, rating: 4.8, reviewCount: 63 },
  "creator-demo": { name: "Avery Chen", initials: "AC", verified: true, rating: 5, reviewCount: 41 },
  "maker-demo": { name: "Makers Common", initials: "MC", verified: true, rating: 4.9, reviewCount: 52 },
  "business-demo": { name: "Alex Rivera", initials: "AR", verified: true, rating: 4.8, reviewCount: 36 },
};

export function sellerFor(ownerId: string) {
  return sellers[ownerId] ?? { name: "BSR Community Member", initials: "BS", verified: true, rating: 4.8, reviewCount: 24 };
}

export const fulfillmentLabel: Record<Fulfillment, string> = {
  pickup: "Pick up",
  delivery: "BSR local delivery",
  owner_location: "Use at owner location",
  on_site: "Use on site",
};

export function fulfillmentOptionsFor(listing: Listing) {
  return listing.fulfillment.map((id) => ({ id, label: fulfillmentLabel[id] }));
}

export function listingPriceLabel(listing: Listing) {
  const price = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(listing.unitPriceCents / 100);
  if (listing.listingType === "sale") return price;
  return `${price} / ${listing.billingUnit === "thirty_minutes" ? "30 min" : "day"}`;
}

export function relatedListings(source: Listing, listings: Listing[], limit = 6) {
  return listings
    .filter((listing) => listing.id !== source.id)
    .map((listing, index) => ({
      listing,
      index,
      score:
        (listing.category === source.category ? 8 : 0) +
        (listing.listingType === source.listingType ? 4 : 0) +
        (listing.city === source.city ? 2 : 0) +
        (Math.abs(listing.unitPriceCents - source.unitPriceCents) <= Math.max(source.unitPriceCents, 2500) ? 1 : 0),
    }))
    .sort((left, right) => right.score - left.score || left.index - right.index)
    .slice(0, limit)
    .map(({ listing }) => listing);
}
