import assert from "node:assert/strict";
import test from "node:test";
import { assistantActions, buildWorkerHandoff, responseForAction } from "./assistant.ts";

test("defines every approved marketplace help action", () => {
  assert.deepEqual(assistantActions.map((action) => action.id), ["rent", "list", "workspace", "delivery", "payment", "worker"]);
});

test("scripted answers explain safety and never request sensitive data", () => {
  assert.match(responseForAction("payment"), /held.*complete/i);
  assert.match(responseForAction("worker"), /do not share/i);
  for (const action of assistantActions) assert.doesNotMatch(responseForAction(action.id), /send (your )?(card|passport|address)/i);
});

test("builds an encoded email handoff when a public destination exists", () => {
  const handoff = buildWorkerHandoff("I need help with PS5 delivery & pickup", "help@bsr.example");
  assert.equal(handoff.mode, "email");
  if (handoff.mode === "email") {
    assert.match(handoff.href, /^mailto:help@bsr\.example\?/);
    assert.match(handoff.href, /PS5%20delivery%20%26%20pickup/);
  }
});

test("does not claim delivery without support configuration", () => {
  assert.deepEqual(buildWorkerHandoff("Please help", ""), { mode: "copy", message: "Please help" });
});
