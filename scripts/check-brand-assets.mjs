import { access, readFile, stat } from "node:fs/promises";
import { constants } from "node:fs";

const requirements = [
  ["apps/web/src/components/BrandLogo.tsx", ["NEXT_PUBLIC_BASE_PATH"]],
  ["apps/runner/src/components/BrandLogo.tsx", ["NEXT_PUBLIC_BASE_PATH"]],
  ["apps/web/src/app/layout.tsx", ["bsr-icon.png"]],
  ["apps/runner/src/app/layout.tsx", ["bsr-icon.png"]],
  ["apps/web/src/app/page.tsx", ["GlobalNav", "HeroSection", "FeaturedListings", "BusinessShowcase", "SiteFooter"]],
  ["apps/web/src/components/GlobalNav.tsx", ["BrandLogo", "backdrop-blur"]],
  ["apps/web/src/components/LoginModal.tsx", ["role=\"dialog\"", "Demo identities"]],
  ["apps/web/src/components/HeroSection.tsx", ["Use more.", "Own less."]],
  ["apps/web/src/components/MarketplaceListingCard.tsx", ["listingPriceLabel", "sellerFor"]],
  ["apps/web/src/components/FeaturedListings.tsx", ["MarketplaceListingCard", "RevealOnScroll"]],
  ["apps/web/src/components/RevealOnScroll.tsx", ["IntersectionObserver", "prefers-reduced-motion"]],
  ["apps/web/src/components/BusinessShowcase.tsx", ["Buy second-hand"]],
  ["apps/web/src/components/SiteFooter.tsx", ["BrandLogo", "Demo marketplace"]],
  ["apps/web/src/components/ShopAssistant.tsx", ["support-launcher-icon", "Open BSR shopping assistant"]],
  ["apps/web/src/app/listings/[id]/page.tsx", ["generateStaticParams", "getDemoListing"]],
  ["apps/web/src/components/ListingDetailView.tsx", ["GlobalNav", "BookingCard", "RelatedListings"]],
  ["apps/web/src/components/ListingGallery.tsx", ["Previous image", "Next image"]],
  ["apps/web/src/components/SellerProfile.tsx", ["Verified", "sellerFor"]],
  ["apps/web/src/components/BookingCard.tsx", ["listing.fulfillment", "Protected quote"]],
  ["apps/web/src/components/RelatedListings.tsx", ["relatedListings", "MarketplaceListingCard"]],
  ["apps/runner/src/components/RunnerNav.tsx", ["BrandLogo", "brand-logo"]],
  ["apps/runner/src/app/page.tsx", ["BrandLogo", "footer-logo"]],
  ["apps/web/src/app/globals.css", ["@import \"tailwindcss\"", ".brand-logo", ".footer-logo"]],
  ["apps/web/next.config.mjs", ["trailingSlash: true"]],
  ["apps/runner/next.config.mjs", ["trailingSlash: true"]],
  ["apps/runner/src/app/globals.css", [".brand-logo", ".footer-logo"]],
];

const assets = [
  "apps/web/public/brand/bsr-icon.svg",
  "apps/web/public/brand/bsr-icon.png",
  "apps/web/public/brand/bsr-hub-logo.svg",
  "apps/runner/public/brand/bsr-icon.svg",
  "apps/runner/public/brand/bsr-icon.png",
  "apps/runner/public/brand/bsr-runner-logo.svg",
];

const failures = [];

for (const [file, markers] of requirements) {
  try {
    const source = await readFile(file, "utf8");
    for (const marker of markers) {
      if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
    }
  } catch {
    failures.push(`${file} does not exist`);
  }
}

for (const asset of assets) {
  try {
    await access(asset, constants.R_OK);
    if ((await stat(asset)).size === 0) failures.push(`${asset} is empty`);
  } catch {
    failures.push(`${asset} does not exist or is unreadable`);
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Brand asset contract passed.");
