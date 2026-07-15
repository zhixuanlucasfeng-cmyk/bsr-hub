import assert from "node:assert/strict";
import test from "node:test";
import { createRunnerStaticDemo } from "./static-demo.ts";

test("static runner quote mirrors the Rust integer-cent formula", async () => {
  const demo = createRunnerStaticDemo();
  const quote = await demo.request("/v1/runner/demo/quote", { method:"POST", body:JSON.stringify({ category:"package_pickup", distance_tenths_mile:32, estimated_minutes:35, weight:"medium", urgency:"same_day", waiting_minutes:0 }) });
  assert.deepEqual(quote, {
    runner_payout_cents: 3033,
    service_fee_cents: 364,
    total_cents: 3397,
    currency: "usd",
    explanation: ["Base pay for package pickup", "Distance allowance for 3.2 miles", "Time allowance for 35 minutes", "medium item · same-day timing", "BSR Runner service fee: 12%"],
  });
});

test("addresses are hidden until the assigned runner accepts", async () => {
  const demo = createRunnerStaticDemo();
  const before = await demo.request<{ pickup_address?: string }>("/v1/runner/demo/tasks/task-1?runner_id=runner-1");
  assert.equal(before.pickup_address, undefined);
  await demo.request("/v1/runner/demo/tasks/task-1/actions", { method:"POST", body:JSON.stringify({ action:"accept", role:"runner", runner_id:"runner-1" }) });
  const after = await demo.request<{ pickup_address?: string }>("/v1/runner/demo/tasks/task-1?runner_id=runner-1");
  assert.equal(after.pickup_address, "1 Fictional Pickup");
});

test("a complete delivery releases payout exactly once", async () => {
  const demo = createRunnerStaticDemo();
  const act = (action: string, role: string, completion_code?: string) => demo.request("/v1/runner/demo/tasks/task-1/actions", { method:"POST", body:JSON.stringify({ action, role, runner_id:"runner-1", completion_code }) });
  await act("accept", "runner"); await act("confirm_pickup", "runner"); await act("start_delivery", "runner"); await act("complete", "customer", "482731");
  const earnings = await demo.request<{ available_cents:number; completed_tasks:number }>("/v1/runner/demo/earnings/runner-1");
  assert.equal(earnings.available_cents, 3033);
  assert.equal(earnings.completed_tasks, 1);
  await assert.rejects(() => act("complete", "customer", "482731"), /not allowed/i);
});

test("prohibited and emergency task quotes are blocked", async () => {
  const demo = createRunnerStaticDemo();
  const input = { distance_tenths_mile:10, estimated_minutes:10, weight:"light", urgency:"flexible", waiting_minutes:0 };
  await assert.rejects(() => demo.request("/v1/runner/demo/quote", { method:"POST", body:JSON.stringify({ ...input, category:"prohibited" }) }), /prohibited/i);
  await assert.rejects(() => demo.request("/v1/runner/demo/quote", { method:"POST", body:JSON.stringify({ ...input, category:"medical_emergency" }) }), /emergency services/i);
});
