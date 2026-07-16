"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { BusinessShowcase } from "../components/BusinessShowcase";
import { CategoryBrowser } from "../components/CategoryBrowser";
import { FeaturedListings } from "../components/FeaturedListings";
import { GlobalNav } from "../components/GlobalNav";
import { HeroSection } from "../components/HeroSection";
import { LoginModal } from "../components/LoginModal";
import { ShopAssistant } from "../components/ShopAssistant";
import { SiteFooter } from "../components/SiteFooter";
import { filterMarketplaceListings, type CategoryId } from "../lib/categories";
import { readMarketplaceEntry } from "../lib/entry-route";
import { demoSessionKey, fulfillmentLabel } from "../lib/marketplace";
import { hubStaticDemo } from "../lib/static-demo";
import { allowedActions, money, personas, type DemoOrder, type Listing } from "../lib/types";

const API = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
const STATIC_DEMO = process.env.NEXT_PUBLIC_STATIC_DEMO === "true";
const actionLabels: Record<string, string> = { mark_paid:"Complete protected payment", confirm:"Seller confirms", activate:"Start rental", fulfill:"Mark fulfilled", return:"Return item", complete:"Complete order", cancel:"Cancel" };
type View = "market" | "orders" | "create";

export default function Home() {
  const [listings, setListings] = useState<Listing[]>([]);
  const [orders, setOrders] = useState<DemoOrder[]>([]);
  const [persona, setPersona] = useState("buyer-demo");
  const [query, setQuery] = useState("");
  const [type, setType] = useState("all");
  const [categoryId, setCategoryId] = useState<CategoryId>("all");
  const [view, setView] = useState<View>("market");
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [loginOpen, setLoginOpen] = useState(false);
  const [notice, setNotice] = useState(STATIC_DEMO ? "Public demo mode — no real payments" : "Rust API connected");
  const pendingAction = useRef<null | (() => void)>(null);
  const activePersona = personas.find((item) => item.id === persona) ?? personas[0];

  const loadOrders = async (id = persona) => {
    if (STATIC_DEMO) { setOrders(hubStaticDemo.ordersFor(id)); return; }
    try { const response = await fetch(`${API}/v1/demo/orders?persona=${id}`); setOrders(await response.json()); }
    catch { setNotice("Rust API offline — run npm run demo"); }
  };

  useEffect(() => {
    try {
      const stored = sessionStorage.getItem(demoSessionKey);
      setSessionId(stored);
      const entry = readMarketplaceEntry(window.location.search);
      const intent = entry.intent;
      setType(entry.listingType);
      if (intent === "orders" || intent === "create") {
        if (stored && stored !== "guest") setView(intent);
        else { pendingAction.current = () => setView(intent); setLoginOpen(true); }
      } else if (!stored) setLoginOpen(true);
      if (stored && stored !== "guest" && personas.some((item) => item.id === stored)) setPersona(stored);
    } catch { setLoginOpen(true); }
  }, []);

  useEffect(() => {
    if (STATIC_DEMO) { setListings(hubStaticDemo.listings()); return; }
    fetch(`${API}/v1/demo/listings`).then((response) => response.json()).then(setListings).catch(() => setNotice("Rust API offline — run npm run demo"));
  }, []);
  useEffect(() => { loadOrders(); }, [persona]);

  const filtered = useMemo(() => filterMarketplaceListings(listings, { query, listingType:type, categoryId }), [listings, type, query, categoryId]);
  const scrollTo = (id: string) => requestAnimationFrame(() => requestAnimationFrame(() => document.getElementById(id)?.scrollIntoView({ behavior:"smooth" })));
  const showMarket = () => { setView("market"); scrollTo("market"); };
  const showCategories = () => { setView("market"); scrollTo("categories"); };
  const selectCategory = (next: CategoryId) => { setCategoryId(next); setView("market"); scrollTo("market"); };

  const requireSession = (action: () => void) => {
    if (sessionId && sessionId !== "guest") { action(); return; }
    pendingAction.current = action;
    setLoginOpen(true);
  };
  const selectDemoIdentity = (id: string) => {
    try { sessionStorage.setItem(demoSessionKey, id); } catch {}
    setSessionId(id); setPersona(id); setLoginOpen(false); setNotice(`Signed in as ${personas.find((item) => item.id === id)?.name ?? "demo user"}.`);
    const action = pendingAction.current; pendingAction.current = null; action?.();
  };
  const browseAsGuest = () => { try { sessionStorage.setItem(demoSessionKey, "guest"); } catch {} setSessionId("guest"); setLoginOpen(false); pendingAction.current = null; };

  const applyBusinessFilter = (listingType: string, category: CategoryId = "all") => { setView("market"); setType(listingType); setCategoryId(category); scrollTo("market"); };
  const act = async (order: DemoOrder, action: string) => {
    if (STATIC_DEMO) {
      try { hubStaticDemo.act(order.id, action); setNotice(`${actionLabels[action]} — validated by the same order rules as the Rust API.`); await loadOrders(); }
      catch { setNotice("That action is not allowed in the current state."); }
      return;
    }
    const response = await fetch(`${API}/v1/demo/orders/${order.id}/actions`, { method:"POST", headers:{ "content-type":"application/json" }, body:JSON.stringify({ action }) });
    if (response.ok) { setNotice(`${actionLabels[action]} — saved by Rust state machine.`); await loadOrders(); }
    else setNotice("That action is not allowed in the current state.");
  };

  return <div className="min-h-screen bg-canvas text-ink">
    <GlobalNav activeView={view} orderCount={orders.length} initials={activePersona.initials} onExplore={() => { setView("market"); window.scrollTo({ top:0, behavior:"smooth" }); }} onCategories={showCategories} onOrders={() => requireSession(() => setView("orders"))} onList={() => requireSession(() => setView("create"))} onAvatar={() => setLoginOpen(true)}/>
    <div className="mt-[76px] bg-zinc-950 px-5 py-2 text-center text-[11px] font-medium text-white/75"><span className="mr-2 inline-block size-2 rounded-full bg-emerald-300"/>{notice}</div>

    {view === "market" && <main>
      <HeroSection onExplore={showMarket} onList={() => requireSession(() => setView("create"))}/>
      <CategoryBrowser selected={categoryId} onSelect={selectCategory}/>
      <FeaturedListings listings={filtered} query={query} listingType={type} onQuery={setQuery} onType={setType}/>
      <BusinessShowcase onRent={() => applyBusinessFilter("rental")} onWorkspace={() => applyBusinessFilter("workspace", "studios")} onSecondHand={() => applyBusinessFilter("sale", "second-hand")}/>
    </main>}

    {view === "orders" && <main className="mx-auto min-h-[70vh] max-w-[1180px] px-5 py-20 sm:px-8">
      <p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Protected transactions</p><h1 className="mt-3 font-sans text-5xl font-bold tracking-tight text-zinc-950">{activePersona.name}&apos;s orders</h1><p className="mt-4 max-w-2xl text-sm leading-6 text-zinc-500">{STATIC_DEMO ? "This public demo mirrors the Rust order rules in your browser." : "The Rust backend decides every valid action."} Switch between Maya and Jordan to complete a rental together.</p>
      {orders.length === 0 ? <div className="mt-10 rounded-card bg-white p-20 text-center text-zinc-500 shadow-soft">No orders yet. Open a listing to begin the protected demo flow.</div> : <div className="mt-10 space-y-5">{orders.map((order) => <article className="rounded-card bg-white p-6 shadow-soft" key={order.id}><div className="flex flex-col gap-5 lg:flex-row lg:items-center"><div className="flex-1"><span className="rounded-full bg-violet-100 px-3 py-1 text-[10px] font-extrabold uppercase text-brand">{order.state.replaceAll("_", " ")}</span><h3 className="mt-3 font-sans text-xl font-bold">{order.listingTitle}</h3><p className="mt-1 text-sm text-zinc-500">{fulfillmentLabel[order.fulfillment]} · {money(order.quote.totalCents)} total</p></div><div className="flex flex-wrap gap-2">{allowedActions(order.state, persona, order).map((action) => <button key={action} className={action === "cancel" ? "rounded-full bg-red-50 px-4 py-2 text-xs font-bold text-red-700" : "rounded-full bg-brand px-4 py-2 text-xs font-bold text-white"} onClick={() => act(order, action)}>{actionLabels[action]}</button>)}</div></div></article>)}</div>}
    </main>}

    {view === "create" && <main className="mx-auto min-h-[70vh] max-w-[920px] px-5 py-20 sm:px-8"><p className="text-xs font-extrabold uppercase tracking-[.16em] text-brand">Any user can earn</p><h1 className="mt-3 font-sans text-5xl font-bold tracking-tight">List an item or space</h1><section className="mt-9 rounded-[24px] bg-white p-6 shadow-soft sm:p-9"><div className="grid gap-5 sm:grid-cols-2"><label className="text-xs font-bold">Listing type<select className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal"><option>Rental product</option><option>Second-hand sale</option><option>Workspace</option></select></label><label className="text-xs font-bold">Title<input className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal" placeholder="What are you offering?"/></label><label className="text-xs font-bold">Category<select className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal"><option>Gaming</option><option>Computers</option><option>Cameras</option><option>Tools</option><option>Studio</option></select></label><label className="text-xs font-bold">Condition<select className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal"><option>Like new</option><option>Excellent</option><option>Good</option><option>Fair</option></select></label><label className="text-xs font-bold sm:col-span-2">Description<textarea className="mt-2 min-h-32 w-full rounded-xl bg-zinc-50 p-3 font-normal" placeholder="Describe condition, included accessories, and rules"/></label><label className="text-xs font-bold">Price in dollars<input type="number" min="1" className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal" placeholder="25"/></label><label className="text-xs font-bold">Refundable deposit<input type="number" min="0" className="mt-2 w-full rounded-xl bg-zinc-50 p-3 font-normal" placeholder="50"/></label></div><div className="mt-6 rounded-xl bg-violet-50 p-4 text-xs leading-5 text-violet-900">Exact addresses stay private. Rust recommends a fair price and limits seller adjustments to ±$5 per billing unit.</div><button className="mt-6 rounded-full bg-gradient-to-r from-brand to-violet-500 px-7 py-3.5 text-sm font-bold text-white" onClick={() => { setNotice("Demo listing saved as a private preview."); setView("market"); }}>Save demo listing</button></section></main>}

    <ShopAssistant onRent={() => applyBusinessFilter("rental")} onList={() => requireSession(() => setView("create"))} onWorkspace={() => applyBusinessFilter("workspace", "studios")} onDelivery={() => applyBusinessFilter("all")}/>
    <LoginModal open={loginOpen} personas={personas} onSelect={selectDemoIdentity} onGuest={browseAsGuest} onClose={() => setLoginOpen(false)}/>
    <SiteFooter/>
  </div>;
}
