import assert from "node:assert/strict";
import test from "node:test";
import { createHubStaticDemo } from "./static-demo.ts";

test("static catalog includes rentals, workspaces, and second-hand sales", () => {
  const demo = createHubStaticDemo();
  const types = new Set(demo.listings().map((listing) => listing.listingType));
  assert.deepEqual(types, new Set(["rental", "workspace", "sale"]));
});

test("quote uses integer cents, a six percent fee, and delivery only when selected", () => {
  const demo = createHubStaticDemo();
  const quote = demo.quote("ps5-slim", 2, "delivery");
  assert.equal(quote.baseCents, 2400);
  assert.equal(quote.serviceFeeCents, 144);
  assert.equal(quote.deliveryFeeCents, 800);
  assert.equal(quote.totalCents, 13344);
});

test("orders are visible to both sides and follow the backend transition rules", () => {
  const demo = createHubStaticDemo();
  const created = demo.createOrder("ps5-slim", 2, "pickup");
  assert.equal(demo.ordersFor("buyer-demo").length, 1);
  assert.equal(demo.ordersFor("seller-demo").length, 1);
  assert.equal(demo.act(created.id, "mark_paid").state, "paid");
  assert.equal(demo.act(created.id, "confirm").state, "confirmed");
  assert.throws(() => demo.act(created.id, "complete"), /not allowed/i);
});

test("invalid units and fulfillment are rejected", () => {
  const demo = createHubStaticDemo();
  assert.throws(() => demo.quote("ps5-slim", 0, "pickup"), /positive/i);
  assert.throws(() => demo.quote("laser-cutter", 1, "delivery"), /unavailable/i);
});
