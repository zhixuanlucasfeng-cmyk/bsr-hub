import assert from "node:assert/strict";
import test from "node:test";
import { allowedActions, money, type DemoOrder } from "./types.ts";

const order = { buyerId: "buyer-demo", sellerId: "seller-demo", state: "paid" } as DemoOrder;
test("money is displayed as US dollars", () => assert.equal(money(13344), "$133.44"));
test("only the seller can confirm a paid order", () => {
  assert.deepEqual(allowedActions("paid", "seller-demo", order), ["confirm", "cancel"]);
  assert.deepEqual(allowedActions("paid", "buyer-demo", order), []);
});
