import assert from "node:assert/strict";
import test from "node:test";
import { categories, filterMarketplaceListings, matchesCategory } from "./categories.ts";
import { createHubStaticDemo } from "./static-demo.ts";

test("defines the seven approved marketplace categories", () => {
  assert.deepEqual(categories.map((item) => item.id), [
    "gaming",
    "computers",
    "cameras",
    "tools",
    "studios",
    "production",
    "second-hand",
  ]);
});

test("matches listings to overlapping marketplace categories", () => {
  const listings = createHubStaticDemo().listings();
  const byId = (id: string) => listings.find((item) => item.id === id)!;

  assert.equal(matchesCategory(byId("ps5-slim"), "gaming"), true);
  assert.equal(matchesCategory(byId("photo-studio"), "studios"), true);
  assert.equal(matchesCategory(byId("print-shop"), "production"), true);
  assert.equal(matchesCategory(byId("monitor-sale"), "second-hand"), true);
  assert.equal(matchesCategory(byId("macbook"), "gaming"), false);
});

test("combines query, listing type, and category filters", () => {
  const listings = createHubStaticDemo().listings();
  const results = filterMarketplaceListings(listings, {
    query: "studio",
    listingType: "workspace",
    categoryId: "studios",
  });

  assert.deepEqual(results.map((item) => item.id), ["photo-studio"]);
});

test("all category keeps matching listings and trims the query", () => {
  const listings = createHubStaticDemo().listings();
  const results = filterMarketplaceListings(listings, {
    query: "  WELLESLEY ",
    listingType: "all",
    categoryId: "all",
  });

  assert.deepEqual(results.map((item) => item.id), ["ps5-slim", "camera-sale"]);
});
