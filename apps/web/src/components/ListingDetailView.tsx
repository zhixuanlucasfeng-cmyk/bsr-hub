"use client";

import { useEffect, useRef, useState } from "react";
import type { Listing } from "../lib/types";
import { personas } from "../lib/types";
import { demoSessionKey, listingPriceLabel } from "../lib/marketplace";
import { BookingCard } from "./BookingCard";
import { GlobalNav } from "./GlobalNav";
import { LinearIcon } from "./LinearIcon";
import { ListingGallery } from "./ListingGallery";
import { LoginModal } from "./LoginModal";
import { RelatedListings } from "./RelatedListings";
import { SellerProfile } from "./SellerProfile";
import { ShopAssistant } from "./ShopAssistant";
import { SiteFooter } from "./SiteFooter";

const basePath = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

export function ListingDetailView({ listing, catalog }: { listing: Listing; catalog: Listing[] }) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [loginOpen, setLoginOpen] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const pendingAction = useRef<null | (() => void)>(null);
  const activePersona = personas.find((persona) => persona.id === sessionId) ?? personas[0];

  useEffect(() => { try { setSessionId(sessionStorage.getItem(demoSessionKey)); } catch {} }, []);
  const home = (destination = "") => { window.location.href = `${basePath}/${destination}`; };
  const protect = (action: () => void) => { if (sessionId && sessionId !== "guest") action(); else { pendingAction.current = action; setLoginOpen(true); } };
  const signIn = (id: string) => { try { sessionStorage.setItem(demoSessionKey, id); } catch {} setSessionId(id); setLoginOpen(false); const action = pendingAction.current; pendingAction.current = null; action?.(); };
  const guest = () => { try { sessionStorage.setItem(demoSessionKey, "guest"); } catch {} setSessionId("guest"); setLoginOpen(false); pendingAction.current = null; };
  const defaultSpecifications = [
    { label:"Condition", value:listing.condition },
    { label:"Category", value:listing.category },
    { label:"Location", value:`${listing.city}, ${listing.state}` },
    { label:"Billing", value:listing.listingType === "sale" ? "One-time purchase" : listing.billingUnit === "day" ? "Per day" : "Per 30 minutes" },
  ];

  return <div className="listing-detail-page min-h-screen bg-canvas pb-20 text-ink lg:pb-0">
    <GlobalNav activeView="market" orderCount={0} initials={activePersona.initials} onExplore={() => home()} onCategories={() => home("#categories")} onOrders={() => protect(() => home("?intent=orders"))} onList={() => protect(() => home("?intent=create"))} onAvatar={() => setLoginOpen(true)}/>
    <div className="mt-[76px] bg-zinc-950 px-5 py-2 text-center text-[11px] font-medium text-white/75"><span className="mr-2 inline-block size-2 rounded-full bg-emerald-300"/>Fictional classroom demo — no real payments or identity data</div>
    <main className="mx-auto max-w-[1320px] px-5 py-10 sm:px-8 lg:px-12 lg:py-16">
      <button onClick={() => home()} className="mb-7 inline-flex items-center gap-2 text-xs font-bold text-zinc-500 transition hover:text-brand"><LinearIcon name="chevron" className="size-4 rotate-180"/>Back to marketplace</button>
      <div className="grid gap-10 lg:grid-cols-[minmax(0,7fr)_minmax(320px,3fr)] lg:items-start">
        <div>
          <ListingGallery listing={listing}/>
          <div className="mt-9"><div className="flex flex-wrap items-center gap-2"><span className="rounded-full bg-violet-100 px-3 py-1.5 text-[10px] font-extrabold uppercase tracking-[.1em] text-brand">{listing.listingType === "workspace" ? "Creative space" : listing.listingType === "sale" ? "Second-hand" : "Rental"}</span><span className="rounded-full bg-zinc-100 px-3 py-1.5 text-[10px] font-bold text-zinc-600">{listing.condition}</span></div><h1 className="mt-4 font-sans text-4xl font-bold leading-tight tracking-[-.04em] text-zinc-950 sm:text-5xl">{listing.title}</h1><p className="mt-3 text-sm font-semibold text-brand">{listingPriceLabel(listing)}</p></div>
          <div className="mt-8"><SellerProfile listing={listing}/></div>
          <section className="mt-8 rounded-card bg-white p-6 shadow-sm sm:p-8"><p className="text-xs font-extrabold uppercase tracking-[.14em] text-brand">About this listing</p><div className={`mt-4 overflow-hidden text-[15px] leading-[1.65] text-zinc-600 transition-all duration-300 ${expanded ? "max-h-96" : "max-h-[4.95em]"}`}>{listing.description} Exact addresses remain private until the protected transaction reaches the correct state. Ask the seller about accessories, setup requirements, and availability before arrival.</div><button onClick={() => setExpanded((value) => !value)} className="mt-3 text-xs font-bold text-brand">{expanded ? "Show less" : "Expand and read more"}</button></section>
          <section className="mt-6 grid gap-6 rounded-card bg-white p-6 shadow-sm sm:grid-cols-2 sm:p-8"><div><p className="text-xs font-extrabold uppercase tracking-[.14em] text-brand">Specifications</p><dl className="mt-4 space-y-3">{(listing.specifications ?? defaultSpecifications).map((item) => <div key={item.label} className="flex justify-between gap-5 border-b border-zinc-100 pb-3 text-xs"><dt className="text-zinc-500">{item.label}</dt><dd className="text-right font-bold text-zinc-900">{item.value}</dd></div>)}</dl></div><div><p className="text-xs font-extrabold uppercase tracking-[.14em] text-brand">Use & protection</p><ul className="mt-4 space-y-3 text-xs leading-5 text-zinc-600">{(listing.usageNotes ?? ["Inspect the item or space before use.", "Keep all communication inside the BSR demo flow.", "Report damage or a dispute before completion."]).map((note) => <li key={note} className="flex gap-2"><LinearIcon name="shield" className="mt-0.5 size-4 shrink-0 text-brand"/>{note}</li>)}</ul></div></section>
        </div>
        <BookingCard listing={listing} onProtectedAction={protect}/>
      </div>
      <RelatedListings listing={listing} listings={catalog}/>
    </main>
    <div className="fixed inset-x-0 bottom-0 z-40 flex items-center justify-between border-t border-zinc-200 bg-white/92 px-5 py-3 shadow-[0_-12px_35px_rgba(20,15,30,.1)] backdrop-blur-xl lg:hidden"><div><small className="block text-[9px] uppercase tracking-wide text-zinc-400">{listing.listingType === "sale" ? "Price" : "From"}</small><strong className="text-sm text-brand">{listingPriceLabel(listing)}</strong></div><button onClick={() => document.getElementById("booking-card")?.scrollIntoView({ behavior:"smooth", block:"center" })} className="rounded-full bg-gradient-to-r from-brand to-violet-500 px-5 py-3 text-xs font-bold text-white">{listing.listingType === "sale" ? "Buy" : listing.listingType === "workspace" ? "Book" : "Reserve"}</button></div>
    <ShopAssistant onRent={() => home("#market")} onList={() => protect(() => home("?intent=create"))} onWorkspace={() => home("#market")} onDelivery={() => home("#market")}/>
    <LoginModal open={loginOpen} personas={personas} onSelect={signIn} onGuest={guest} onClose={() => setLoginOpen(false)}/>
    <SiteFooter/>
  </div>;
}
