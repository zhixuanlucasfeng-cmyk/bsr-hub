import { useMemo, useState } from "react";
import type { RunnerAction, RunnerPersona, RunnerTask } from "../lib/contracts";
import { actionLabel, actionsFor, money, taskProgress } from "../lib/contracts";

interface TaskMarketViewProps {
  tasks: RunnerTask[];
  persona: RunnerPersona;
  onSelect: (task: RunnerTask) => void;
}

export function TaskMarketView({ tasks, persona, onSelect }: TaskMarketViewProps) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("all");
  const filtered = useMemo(() => tasks.filter((task) => (category === "all" || task.category === category) && `${task.title} ${task.pickup_area} ${task.dropoff_area}`.toLowerCase().includes(query.toLowerCase())), [tasks, query, category]);
  return (
    <main className="market-page">
      <div className="page-intro market-intro"><div><p className="eyebrow">LOCAL TASK MARKET</p><h1>{persona === "runner" ? "Choose work that fits your day." : "See what is moving nearby."}</h1><p>Public cards never reveal exact pickup or delivery addresses.</p></div><div className="market-stat"><strong>{filtered.filter((task) => task.state === "available").length}</strong><span>jobs open now</span></div></div>
      <div className="filters"><input aria-label="Search jobs" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search jobs or neighborhoods…"/><select aria-label="Task category" value={category} onChange={(e) => setCategory(e.target.value)}><option value="all">All task types</option><option value="package_pickup">Package pickup</option><option value="grocery_pickup">Grocery pickup</option><option value="document_delivery">Documents</option><option value="bsr_rental_delivery">BSR deliveries</option></select></div>
      <div className="market-grid">{filtered.map((task) => <article className="market-card" key={task.id}><div className="job-card-top"><span className="category-icon">{task.category.includes("grocery") ? "🛍" : task.category.includes("document") ? "✉" : task.category.includes("rental") ? "📷" : "📦"}</span><span className={`state ${task.state}`}>{task.state.replaceAll("_", " ")}</span></div><p className="route-label">⌖ {task.pickup_area} <b>→</b> {task.dropoff_area}</p><h2>{task.title}</h2><p className="task-description">{task.description}</p><div className="job-facts"><span>{(task.distance_tenths_mile / 10).toFixed(1)} miles</span><span>{task.estimated_minutes} min</span><span>{task.weight}</span><span>{task.urgency.replaceAll("_", " ")}</span></div><div className="market-card-footer"><div><small>Runner payout</small><strong>{money(task.runner_payout_cents)}</strong></div><button onClick={() => onSelect(task)}>View task →</button></div></article>)}</div>
      {!filtered.length && <div className="empty-state"><span>⌕</span><h2>No matching jobs</h2><p>Try a broader search or another category.</p></div>}
    </main>
  );
}

interface TaskTrackingViewProps {
  task: RunnerTask;
  persona: RunnerPersona;
  busy: boolean;
  onBack: () => void;
  onAction: (action: RunnerAction, completionCode?: string) => void;
}

const milestones = ["available", "accepted", "picked_up", "delivering", "completed"];

export function TaskTrackingView({ task, persona, busy, onBack, onAction }: TaskTrackingViewProps) {
  const [completionCode, setCompletionCode] = useState("482731");
  const actions = actionsFor(task.state, persona);
  return (
    <main className="tracking-page">
      <button className="back-button" onClick={onBack}>← Back to jobs</button>
      <div className="tracking-layout">
        <section className="tracking-main"><div className="tracking-heading"><div><p className="eyebrow">TASK {task.id.toUpperCase()}</p><h1>{task.title}</h1></div><span className={`state large ${task.state}`}>{task.state.replaceAll("_", " ")}</span></div><div className="progress-track"><div style={{ width: `${taskProgress(task.state)}%` }}/></div><div className="milestones">{milestones.map((state) => <span className={taskProgress(task.state) >= taskProgress(state as RunnerTask["state"]) ? "reached" : ""} key={state}><i/>{state.replaceAll("_", " ")}</span>)}</div>
          <div className="route-panel"><div><span className="route-dot pickup-dot">A</span><small>Pickup area</small><strong>{task.pickup_area}</strong>{task.pickup_address && <p>{task.pickup_address}</p>}</div><div className="route-line"/><div><span className="route-dot dropoff-dot">B</span><small>Drop-off area</small><strong>{task.dropoff_area}</strong>{task.dropoff_address && <p>{task.dropoff_address}</p>}</div></div>
          {!task.pickup_address && <p className="privacy-note">🔒 Exact addresses remain protected until an approved runner accepts this task.</p>}
          <div className="task-detail-grid"><div><small>Distance</small><strong>{(task.distance_tenths_mile / 10).toFixed(1)} miles</strong></div><div><small>Estimated time</small><strong>{task.estimated_minutes} minutes</strong></div><div><small>Weight</small><strong>{task.weight}</strong></div><div><small>Urgency</small><strong>{task.urgency.replaceAll("_", " ")}</strong></div></div>
        </section>
        <aside className="action-panel"><p className="eyebrow">PROTECTED TASK</p><h2>Next action</h2><p>Rust allows only actions valid for the current role and state.</p><div className="payout-box"><small>Runner payout</small><strong>{money(task.runner_payout_cents)}</strong><span>Total held: {money(task.total_cents)}</span></div>{actions.includes("complete") && <label>Completion code<input aria-label="Completion code" value={completionCode} onChange={(e) => setCompletionCode(e.target.value)} maxLength={6}/><small>Classroom demo code: 482731</small></label>}<div className="action-stack">{actions.map((action) => <button className={action === "dispute" || action === "cancel" ? "danger-button" : "primary"} disabled={busy} onClick={() => onAction(action, action === "complete" ? completionCode : undefined)} key={action}>{actionLabel[action]}</button>)}</div>{!actions.length && <div className="done-message"><span>✓</span><strong>{task.state === "completed" ? "Task completed" : "No action needed"}</strong><p>{task.state === "completed" ? "The simulated payout has been released." : "Switch demo persona to continue this journey."}</p></div>}</aside>
      </div>
    </main>
  );
}
