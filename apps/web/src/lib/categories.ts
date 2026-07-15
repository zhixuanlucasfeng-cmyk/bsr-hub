import type { Listing } from "./types.ts";

export type CategoryId = "all" | "gaming" | "computers" | "cameras" | "tools" | "studios" | "production" | "second-hand";
export type BrowseCategoryId = Exclude<CategoryId, "all">;

export interface CategoryDefinition {
  id: BrowseCategoryId;
  label: string;
  example: string;
  imageSrc: string;
}

export interface MarketplaceFilters {
  query: string;
  listingType: string;
  categoryId: CategoryId;
}

export const categories: CategoryDefinition[] = [
  { id:"gaming", label:"Gaming & consoles", example:"PS5 systems and controllers", imageSrc:"/images/categories/gaming.jpg" },
  { id:"computers", label:"Computers & electronics", example:"Laptops, monitors, and PCs", imageSrc:"/images/categories/computers.jpg" },
  { id:"cameras", label:"Cameras & creator gear", example:"Cameras, audio, and podcast kits", imageSrc:"/images/categories/cameras.jpg" },
  { id:"tools", label:"Tools & maker equipment", example:"Project tools and fabrication gear", imageSrc:"/images/categories/tools.jpg" },
  { id:"studios", label:"Studios & workspaces", example:"Photo, podcast, and maker studios", imageSrc:"/images/categories/studios.jpg" },
  { id:"production", label:"Small production spaces", example:"Printing and small-batch workshops", imageSrc:"/images/categories/production.jpg" },
  { id:"second-hand", label:"Second-hand goods", example:"Useful products with protected payment", imageSrc:"/images/categories/second-hand.jpg" },
];

export function matchesCategory(listing: Listing, categoryId: CategoryId): boolean {
  if (categoryId === "all") return true;
  if (categoryId === "second-hand") return listing.listingType === "sale";

  const category = listing.category.toLowerCase();
  if (categoryId === "gaming") return category === "gaming";
  if (categoryId === "computers") return category === "computers";
  if (categoryId === "cameras") return category === "cameras";
  if (categoryId === "tools") return category === "tools";
  if (categoryId === "studios") return listing.listingType === "workspace";
  return category === "maker space" || category === "printing";
}

export function filterMarketplaceListings(listings: Listing[], filters: MarketplaceFilters): Listing[] {
  const needle = filters.query.trim().toLowerCase();
  return listings.filter((listing) =>
    (filters.listingType === "all" || listing.listingType === filters.listingType) &&
    matchesCategory(listing, filters.categoryId) &&
    (!needle || `${listing.title} ${listing.category} ${listing.city}`.toLowerCase().includes(needle))
  );
}
