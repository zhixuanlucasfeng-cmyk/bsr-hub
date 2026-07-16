import assert from "node:assert/strict";
import test from "node:test";
import { demoCatalog } from "./static-demo.ts";
import { fulfillmentOptionsFor, listingPriceLabel, relatedListings, sellerFor } from "./marketplace.ts";

test("related listings prefer the same category and type without returning the source", () => {
  const source = demoCatalog.find((listing) => listing.id === "ps5-slim")!;
  const related = relatedListings(source, demoCatalog, 4);

  assert.equal(related.length, 4);
  assert.equal(related.some((listing) => listing.id === source.id), false);
  assert.equal(related[0]?.id, "ps5-pro");
});

test("related listings stop at the requested limit", () => {
  const source = demoCatalog[0]!;
  assert.equal(relatedListings(source, demoCatalog, 2).length, 2);
});

test("listing price labels distinguish rentals, workspaces, and sales", () => {
  assert.equal(listingPriceLabel(demoCatalog.find((listing) => listing.id === "ps5-slim")!), "$12.00 / 30 min");
  assert.equal(listingPriceLabel(demoCatalog.find((listing) => listing.id === "photo-studio")!), "$35.00 / 30 min");
  assert.equal(listingPriceLabel(demoCatalog.find((listing) => listing.id === "monitor-sale")!), "$165.00");
});

test("workspace fulfillment remains on site and never invents delivery", () => {
  const workspace = demoCatalog.find((listing) => listing.id === "photo-studio")!;
  assert.deepEqual(fulfillmentOptionsFor(workspace), [{ id: "on_site", label: "Use on site" }]);
});

test("seller metadata is deterministic for an owner", () => {
  assert.deepEqual(sellerFor("seller-demo"), sellerFor("seller-demo"));
  assert.equal(sellerFor("seller-demo").verified, true);
});
