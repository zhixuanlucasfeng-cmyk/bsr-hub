import assert from "node:assert/strict";
import { test } from "node:test";
import { describeAuthGate } from "./auth-state.ts";

test("auth gate reports unavailable when online auth is not configured", () => {
  assert.equal(
    describeAuthGate({ authConfigured: false, signedIn: false }),
    "api-unavailable",
  );
});

test("auth gate reports signed out when auth exists but no session exists", () => {
  assert.equal(
    describeAuthGate({ authConfigured: true, signedIn: false }),
    "signed-out",
  );
});

test("auth gate reports signed in when auth exists and a session exists", () => {
  assert.equal(
    describeAuthGate({ authConfigured: true, signedIn: true }),
    "signed-in",
  );
});
