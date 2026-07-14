import type { RunnerPersona } from "../lib/contracts";

export type Screen = "home" | "post" | "market" | "apply" | "tracking" | "earnings" | "admin";

interface RunnerNavProps {
  persona: RunnerPersona;
  screen: Screen;
  taskCount: number;
  onPersona: (persona: RunnerPersona) => void;
  onScreen: (screen: Screen) => void;
  onReset: () => void;
}

export function RunnerNav({ persona, screen, taskCount, onPersona, onScreen, onReset }: RunnerNavProps) {
  return (
    <header className="topbar">
      <button className="brand" onClick={() => onScreen("home")} aria-label="BSR Runner home">
        <span className="brand-mark">B</span>
        <span>BSR <strong>Runner</strong></span>
      </button>
      <nav aria-label="Primary navigation">
        <button className={screen === "market" ? "active" : ""} onClick={() => onScreen("market")}>Jobs <span>{taskCount}</span></button>
        {persona === "customer" && <button className={screen === "post" ? "active" : ""} onClick={() => onScreen("post")}>Post a task</button>}
        {persona === "runner" && <button className={screen === "earnings" ? "active" : ""} onClick={() => onScreen("earnings")}>Earnings</button>}
        {persona === "admin" && <button className={screen === "admin" ? "active" : ""} onClick={() => onScreen("admin")}>Safety desk</button>}
      </nav>
      <div className="nav-tools">
        <label className="persona-label">
          Demo as
          <select value={persona} onChange={(event) => onPersona(event.target.value as RunnerPersona)}>
            <option value="customer">Maya · Customer</option>
            <option value="runner">Jordan · Runner</option>
            <option value="admin">Alex · Admin</option>
          </select>
        </label>
        <button className="avatar-button" onClick={onReset} title="Reset fictional demo data">{persona === "customer" ? "MB" : persona === "runner" ? "JS" : "AD"}</button>
      </div>
    </header>
  );
}
