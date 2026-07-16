export type EntryIntent = "orders" | "create" | null;
export type EntryListingType = "all" | "rental" | "workspace" | "sale";

export interface MarketplaceEntry {
  intent: EntryIntent;
  listingType: EntryListingType;
}

export function readMarketplaceEntry(search: string): MarketplaceEntry {
  const params = new URLSearchParams(search);
  const rawIntent = params.get("intent");
  const rawType = params.get("type");
  const intent: EntryIntent = rawIntent === "orders" || rawIntent === "create" ? rawIntent : null;
  const listingType: EntryListingType = rawType === "rental" || rawType === "workspace" || rawType === "sale" ? rawType : "all";
  return { intent, listingType };
}
