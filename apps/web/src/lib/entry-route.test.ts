import assert from "node:assert/strict";
import test from "node:test";
import { readMarketplaceEntry } from "./entry-route.ts";

test("reads a valid footer listing filter", () => {
  assert.deepEqual(readMarketplaceEntry("?type=rental"), { intent:null, listingType:"rental" });
  assert.deepEqual(readMarketplaceEntry("?type=workspace"), { intent:null, listingType:"workspace" });
  assert.deepEqual(readMarketplaceEntry("?type=sale"), { intent:null, listingType:"sale" });
});

test("reads protected direct-entry intents", () => {
  assert.deepEqual(readMarketplaceEntry("?intent=orders"), { intent:"orders", listingType:"all" });
  assert.deepEqual(readMarketplaceEntry("?intent=create"), { intent:"create", listingType:"all" });
});

test("rejects unsupported query values", () => {
  assert.deepEqual(readMarketplaceEntry("?type=unknown&intent=delete"), { intent:null, listingType:"all" });
});
