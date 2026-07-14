"use client";

import { useCallback, useEffect, useState } from "react";
import { ApplicationView } from "../components/ApplicationView";
import { AdminView, EarningsView } from "../components/DashboardViews";
import { HomeView } from "../components/HomeView";
import { PostTaskView, type TaskDraft } from "../components/PostTaskView";
import { RunnerNav, type Screen } from "../components/RunnerNav";
import { TaskMarketView, TaskTrackingView } from "../components/TaskMarketView";
import type { AdminSummary, RunnerAction, RunnerApplication, RunnerEarnings, RunnerPersona, RunnerQuote, RunnerTask } from "../lib/contracts";

const API = process.env.NEXT_PUBLIC_RUNNER_API_URL ?? "http://localhost:8080";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API}${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  if (!response.ok) {
    const body = await response.json().catch(() => ({ message: "The demo service could not complete that action." }));
    throw new Error(body.message ?? body.error?.message ?? `Request failed (${response.status})`);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export default function RunnerDemo() {
  const [screen, setScreen] = useState<Screen>("home");
  const [persona, setPersona] = useState<RunnerPersona>("customer");
  const [tasks, setTasks] = useState<RunnerTask[]>([]);
  const [selected, setSelected] = useState<RunnerTask | null>(null);
  const [applications, setApplications] = useState<RunnerApplication[]>([]);
  const [earnings, setEarnings] = useState<RunnerEarnings | null>(null);
  const [summary, setSummary] = useState<AdminSummary | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("Fictional classroom demo — no real payments or identity data.");

  const loadTasks = useCallback(async () => setTasks(await request<RunnerTask[]>("/v1/runner/demo/tasks")), []);
  const loadDashboards = useCallback(async () => {
    const [nextApplications, nextEarnings, nextSummary] = await Promise.all([
      request<RunnerApplication[]>("/v1/runner/demo/applications"),
      request<RunnerEarnings>("/v1/runner/demo/earnings/runner-1"),
      request<AdminSummary>("/v1/runner/demo/admin"),
    ]);
    setApplications(nextApplications); setEarnings(nextEarnings); setSummary(nextSummary);
  }, []);

  useEffect(() => { loadTasks().catch((error) => setNotice(error.message)); }, [loadTasks]);
  useEffect(() => { if (screen === "admin" || screen === "earnings") loadDashboards().catch((error) => setNotice(error.message)); }, [screen, loadDashboards]);

  const openTask = async (task: RunnerTask) => {
    try {
      const query = persona === "runner" ? "?runner_id=runner-1" : "";
      setSelected(await request<RunnerTask>(`/v1/runner/demo/tasks/${task.id}${query}`));
      setScreen("tracking");
    } catch (error) { setNotice(error instanceof Error ? error.message : "Could not open task"); }
  };

  const act = async (action: RunnerAction, completionCode?: string) => {
    if (!selected) return;
    setBusy(true);
    try {
      await request(`/v1/runner/demo/tasks/${selected.id}/actions`, { method: "POST", body: JSON.stringify({ action, role: persona, runner_id: "runner-1", completion_code: completionCode }) });
      await loadTasks();
      const query = persona === "runner" ? "?runner_id=runner-1" : "";
      setSelected(await request<RunnerTask>(`/v1/runner/demo/tasks/${selected.id}${query}`));
      setNotice(action === "complete" ? "Task completed — protected demo payout released." : "Task state updated by the Rust rules engine.");
    } catch (error) { setNotice(error instanceof Error ? error.message : "Action failed"); }
    finally { setBusy(false); }
  };

  const publish = async (draft: TaskDraft) => {
    const task = await request<RunnerTask>("/v1/runner/demo/tasks", { method: "POST", body: JSON.stringify(draft) });
    await request(`/v1/runner/demo/tasks/${task.id}/actions`, { method: "POST", body: JSON.stringify({ action: "fund", role: "customer" }) });
    const published = await request<RunnerTask>(`/v1/runner/demo/tasks/${task.id}/actions`, { method: "POST", body: JSON.stringify({ action: "publish", role: "customer" }) });
    await loadTasks();
    return published;
  };

  const approve = async (id: string) => {
    const application = await request<RunnerApplication>(`/v1/runner/demo/applications/${id}/approve`, { method: "POST" });
    await loadDashboards();
    return application;
  };

  const reset = async () => {
    await request<void>("/v1/runner/demo/reset", { method: "POST" });
    setSelected(null); setScreen("home"); setPersona("customer");
    await Promise.all([loadTasks(), loadDashboards()]);
    setNotice("Demo data reset. Start the full journey again.");
  };

  const changePersona = (next: RunnerPersona) => {
    setPersona(next); setSelected(null);
    setScreen(next === "admin" ? "admin" : next === "runner" ? "market" : "home");
  };

  return <div className="runner-shell">
    <RunnerNav persona={persona} screen={screen} taskCount={tasks.filter((task) => task.state === "available").length} onPersona={changePersona} onScreen={setScreen} onReset={() => reset().catch((error) => setNotice(error.message))}/>
    <div className="demo-ribbon" role="status"><span>DEMO</span>{notice}</div>
    {screen === "home" && <HomeView tasks={tasks} onScreen={setScreen}/>} 
    {screen === "post" && <PostTaskView onQuote={(draft) => request<RunnerQuote>("/v1/runner/demo/quote", { method: "POST", body: JSON.stringify(draft) })} onPublish={publish} onCreated={(task) => { setSelected(task); setScreen("tracking"); setNotice("Task funded and published to local runners."); }}/>} 
    {screen === "market" && <TaskMarketView tasks={tasks} persona={persona} onSelect={openTask}/>} 
    {screen === "apply" && <ApplicationView onApply={(input) => request<RunnerApplication>("/v1/runner/demo/applications", { method: "POST", body: JSON.stringify(input) })} onApprove={approve}/>} 
    {screen === "tracking" && selected && <TaskTrackingView task={selected} persona={persona} busy={busy} onBack={() => setScreen("market")} onAction={act}/>} 
    {screen === "earnings" && <EarningsView earnings={earnings} tasks={tasks}/>} 
    {screen === "admin" && <AdminView summary={summary} applications={applications} tasks={tasks} onApprove={(id) => { approve(id).catch((error) => setNotice(error.message)); }}/>} 
    <footer><div className="footer-brand"><span className="brand-mark">B</span><div><strong>BSR Runner</strong><small>A BSR Hub classroom venture</small></div></div><p>Fair local tasks. Protected payments. Dignified flexible work.</p><div><span>Fictional demo</span><span>Babson Summer Study</span><span>SDG 8 & 10</span></div></footer>
  </div>;
}
