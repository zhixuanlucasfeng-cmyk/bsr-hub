import type { RunnerTask } from "../lib/contracts";
import { money } from "../lib/contracts";
import type { Screen } from "./RunnerNav";

interface HomeViewProps {
  tasks: RunnerTask[];
  onScreen: (screen: Screen) => void;
}

const steps = [
  ["01", "Post or find a task", "Customers describe a lawful local errand. Runners browse work that fits their time and transport."],
  ["02", "BSR calculates fair pay", "Distance, time, weight and urgency create an explainable price — no bidding people down."],
  ["03", "Finish with protection", "Payment stays protected until pickup, delivery and a customer completion code are confirmed."],
];

export function HomeView({ tasks, onScreen }: HomeViewProps) {
  return (
    <>
      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">LOCAL HELP · FLEXIBLE EARNINGS</p>
          <h1>Get things moving.<br/><em>Earn on your terms.</em></h1>
          <p className="hero-lede">BSR Runner connects neighbors who need a hand with adults ready to earn through safe, clearly priced local tasks.</p>
          <div className="hero-actions">
            <button className="primary" onClick={() => onScreen("post")}>Post a task <span>→</span></button>
            <button className="secondary" onClick={() => onScreen("apply")}>Become a runner</button>
          </div>
          <div className="trust-row"><span>✓ Protected payment</span><span>✓ Automatic fair pricing</span><span>✓ Address privacy</span></div>
        </div>
        <div className="hero-art" aria-label="Local delivery illustration">
          <div className="route route-one"/><div className="route route-two"/>
          <div className="map-card pickup"><span>1</span><strong>Pickup</strong><small>Wellesley Square</small></div>
          <div className="map-card dropoff"><span>2</span><strong>Drop-off</strong><small>Babson Park</small></div>
          <div className="runner-orbit"><div className="runner-icon">🚲</div></div>
          <div className="earnings-pill"><small>Estimated runner pay</small><strong>$29.33</strong></div>
        </div>
      </section>

      <section className="impact-strip">
        <div><strong>18+</strong><span>verified-age rule</span></div>
        <div><strong>12%</strong><span>transparent service fee</span></div>
        <div><strong>100%</strong><span>fictional classroom data</span></div>
        <div><strong>SDG 8</strong><span>decent work & growth</span></div>
      </section>

      <section className="section how">
        <p className="eyebrow">HOW IT WORKS</p>
        <div className="section-heading"><h2>Simple for customers.<br/>Fair for runners.</h2><p>No bidding wars, hidden addresses or cash handoffs. Rust rules keep each task moving safely.</p></div>
        <div className="step-grid">{steps.map(([number, title, text]) => <article key={number}><span>{number}</span><h3>{title}</h3><p>{text}</p></article>)}</div>
      </section>

      <section className="section live-jobs">
        <div className="section-heading"><div><p className="eyebrow">LIVE NEAR BABSON</p><h2>Work available now</h2></div><button className="text-button" onClick={() => onScreen("market")}>See every job →</button></div>
        <div className="job-preview-grid">{tasks.slice(0, 3).map((task) => <article className="job-card" key={task.id}><div className="job-card-top"><span className="category-icon">{task.category.includes("grocery") ? "🛍" : task.category.includes("document") ? "✉" : "📦"}</span><span className={`state ${task.state}`}>{task.state.replaceAll("_", " ")}</span></div><p className="route-label">{task.pickup_area} → {task.dropoff_area}</p><h3>{task.title}</h3><div className="job-facts"><span>{(task.distance_tenths_mile / 10).toFixed(1)} mi</span><span>{task.estimated_minutes} min</span><span>{task.weight}</span></div><div className="job-payout"><small>Runner earns</small><strong>{money(task.runner_payout_cents)}</strong></div></article>)}</div>
      </section>

      <section className="safety-callout"><div><p className="eyebrow">BUILT FOR TRUST</p><h2>Local opportunity without cutting corners.</h2><p>BSR Runner blocks dangerous or illegal requests, keeps addresses private until acceptance, and records every protected state change.</p></div><div className="shield">✦<span>Safety<br/>first</span></div></section>
    </>
  );
}
