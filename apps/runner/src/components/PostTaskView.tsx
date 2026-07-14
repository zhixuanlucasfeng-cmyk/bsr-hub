import { FormEvent, useState } from "react";
import type { RunnerQuote, RunnerTask, TaskCategory } from "../lib/contracts";
import { money } from "../lib/contracts";

export interface TaskDraft {
  customer_id: string;
  title: string;
  description: string;
  pickup_area: string;
  dropoff_area: string;
  pickup_address: string;
  dropoff_address: string;
  category: TaskCategory;
  distance_tenths_mile: number;
  estimated_minutes: number;
  weight: "light" | "medium" | "heavy";
  urgency: "flexible" | "same_day" | "immediate";
  waiting_minutes: number;
  safety_confirmed: boolean;
}

interface PostTaskViewProps {
  onQuote: (draft: TaskDraft) => Promise<RunnerQuote>;
  onPublish: (draft: TaskDraft) => Promise<RunnerTask>;
  onCreated: (task: RunnerTask) => void;
}

const initial: TaskDraft = { customer_id: "customer-1", title: "", description: "", pickup_area: "Wellesley Square", dropoff_area: "Babson Park", pickup_address: "12 Fictional Pickup Street", dropoff_address: "99 Demo Dropoff Avenue", category: "package_pickup", distance_tenths_mile: 32, estimated_minutes: 35, weight: "medium", urgency: "same_day", waiting_minutes: 0, safety_confirmed: false };

export function PostTaskView({ onQuote, onPublish, onCreated }: PostTaskViewProps) {
  const [draft, setDraft] = useState<TaskDraft>(initial);
  const [quote, setQuote] = useState<RunnerQuote | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const set = <K extends keyof TaskDraft>(key: K, value: TaskDraft[K]) => { setDraft((current) => ({ ...current, [key]: value })); setQuote(null); };
  const quoteTask = async (event: FormEvent) => { event.preventDefault(); setBusy(true); setError(""); try { setQuote(await onQuote(draft)); } catch (value) { setError(value instanceof Error ? value.message : "Could not calculate this task."); } finally { setBusy(false); } };
  const publish = async () => { setBusy(true); setError(""); try { onCreated(await onPublish(draft)); } catch (value) { setError(value instanceof Error ? value.message : "Could not publish this task."); } finally { setBusy(false); } };
  return (
    <main className="form-page">
      <div className="page-intro"><p className="eyebrow">CUSTOMER TASK BUILDER</p><h1>What needs moving?</h1><p>Describe a lawful local errand. BSR Runner calculates fair pay automatically.</p></div>
      <div className="form-layout">
        <form className="task-form" onSubmit={quoteTask}>
          <div className="form-section"><span className="form-number">1</span><div><h2>Task basics</h2><p>Public information runners see before accepting.</p></div></div>
          <label>Task type<select value={draft.category} onChange={(e) => set("category", e.target.value as TaskCategory)}><option value="package_pickup">Package pickup</option><option value="grocery_pickup">Grocery pickup</option><option value="document_delivery">Document delivery</option><option value="small_item_delivery">Small-item delivery</option><option value="bsr_rental_delivery">BSR rental delivery</option><option value="bsr_second_hand_delivery">BSR second-hand delivery</option><option value="other_errand">Other lawful errand</option><option value="prohibited">Prohibited item — safety test</option><option value="medical_emergency">Medical emergency — safety test</option></select></label>
          <label>Short title<input required value={draft.title} onChange={(e) => set("title", e.target.value)} placeholder="Pick up a package and deliver it nearby"/></label>
          <label>Details<textarea required value={draft.description} onChange={(e) => set("description", e.target.value)} placeholder="Describe the item, access needs and what the runner should expect."/></label>
          <div className="form-section"><span className="form-number">2</span><div><h2>Route</h2><p>Only general areas appear publicly.</p></div></div>
          <div className="field-pair"><label>Pickup area<input required value={draft.pickup_area} onChange={(e) => set("pickup_area", e.target.value)}/></label><label>Drop-off area<input required value={draft.dropoff_area} onChange={(e) => set("dropoff_area", e.target.value)}/></label></div>
          <div className="field-pair private-fields"><label>Private pickup address<input required value={draft.pickup_address} onChange={(e) => set("pickup_address", e.target.value)}/></label><label>Private drop-off address<input required value={draft.dropoff_address} onChange={(e) => set("dropoff_address", e.target.value)}/></label></div>
          <p className="privacy-note">🔒 Exact addresses are revealed only to the assigned runner.</p>
          <div className="field-triple"><label>Distance (miles)<input type="number" min="0.1" max="100" step="0.1" value={draft.distance_tenths_mile / 10} onChange={(e) => set("distance_tenths_mile", Math.round(Number(e.target.value) * 10))}/></label><label>Estimated minutes<input type="number" min="5" max="480" value={draft.estimated_minutes} onChange={(e) => set("estimated_minutes", Number(e.target.value))}/></label><label>Waiting minutes<input type="number" min="0" max="120" value={draft.waiting_minutes} onChange={(e) => set("waiting_minutes", Number(e.target.value))}/></label></div>
          <div className="field-pair"><label>Weight<select value={draft.weight} onChange={(e) => set("weight", e.target.value as TaskDraft["weight"])}><option value="light">Light · under 5 lb</option><option value="medium">Medium · 5–20 lb</option><option value="heavy">Heavy · 20–40 lb</option></select></label><label>Timing<select value={draft.urgency} onChange={(e) => set("urgency", e.target.value as TaskDraft["urgency"])}><option value="flexible">Flexible</option><option value="same_day">Same day</option><option value="immediate">Immediate</option></select></label></div>
          <label className="check-row"><input type="checkbox" checked={draft.safety_confirmed} onChange={(e) => set("safety_confirmed", e.target.checked)}/><span>I confirm this is lawful, non-emergency work with no dangerous goods, cash transfer, controlled substances, weapons or private-residence entry.</span></label>
          {error && <p className="form-error" role="alert">{error}</p>}
          <button className="primary wide" disabled={busy || !draft.safety_confirmed}>{busy ? "Calculating…" : "Get automatic quote →"}</button>
        </form>
        <aside className="quote-panel">
          <p className="eyebrow">RUST PRICE ENGINE</p><h2>{quote ? "Your protected quote" : "Fair pay, explained"}</h2>
          {quote ? <><div className="quote-total"><small>Total protected payment</small><strong>{money(quote.total_cents)}</strong></div><dl><div><dt>Runner earns</dt><dd>{money(quote.runner_payout_cents)}</dd></div><div><dt>BSR service fee</dt><dd>{money(quote.service_fee_cents)}</dd></div></dl><ul>{quote.explanation.map((line) => <li key={line}>{line}</li>)}</ul><button type="button" className="primary wide" disabled={busy} onClick={publish}>{busy ? "Publishing…" : "Fund & publish demo task"}</button><p className="demo-caption">No real card is charged in this classroom demo.</p></> : <><div className="formula"><span>Distance</span><b>+</b><span>Time</span><b>+</b><span>Weight</span><b>+</b><span>Urgency</span></div><p>Customers cannot push payment below the automatic minimum. Runners see their payout before accepting.</p></>}
        </aside>
      </div>
    </main>
  );
}
