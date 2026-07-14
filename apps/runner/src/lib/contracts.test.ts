import assert from "node:assert/strict";
import test from "node:test";

import { actionsFor, money, taskProgress } from "./contracts.ts";

test("money formats integer cents as US dollars", () => {
  assert.equal(money(1845), "$18.45");
});

test("only a runner accepts an available task", () => {
  assert.deepEqual(actionsFor("available", "runner"), ["accept"]);
  assert.deepEqual(actionsFor("available", "customer"), ["cancel"]);
});

test("delivery completion belongs to the customer", () => {
  assert.deepEqual(actionsFor("delivering", "customer"), ["complete", "dispute"]);
  assert.deepEqual(actionsFor("delivering", "runner"), ["dispute"]);
});

test("progress increases through the successful delivery states", () => {
  assert.equal(taskProgress("available"), 25);
  assert.equal(taskProgress("picked_up"), 70);
  assert.equal(taskProgress("completed"), 100);
});
