"use client";

import { useState } from "react";
import type { Fulfillment, Listing, Quote } from "../lib/types";
import { money } from "../lib/types";
import { fulfillmentLabel, listingPriceLabel } from "../lib/marketplace";
import { hubStaticDemo } from "../lib/static-demo";
import { LinearIcon } from "./LinearIcon";

const API = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
const STATIC_DEMO = process.env.NEXT_PUBLIC_STATIC_DEMO === "true";

interface BookingCardProps { listing: Listing; onProtectedAction: (action: () => void) => void }

export function BookingCard({ listing, onProtectedAction }: BookingCardProps) {
  const [units, setUnits] = useState(listing.listingType === "sale" ? 1 : listing.billingUnit === "thirty_minutes" ? 2 : 1);
  const [fulfillment, setFulfillment] = useState<Fulfillment>(listing.fulfillment[0]);
  const [quote, setQuote] = useState<Quote | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const actionLabel = listing.listingType === "sale" ? "Buy now" : listing.listingType === "workspace" ? "Book this space" : "Reserve rental";

  const getQuote = async () => {
    setBusy(true); setStatus("");
    try {
      if (STATIC_DEMO) setQuote(hubStaticDemo.quote(listing.id, units, fulfillment));
      else {
        const response = await fetch(`${API}/v1/demo/quote`, { method:"POST", headers:{ "content-type":"application/json" }, body:JSON.stringify({ listingId:listing.id, units, fulfillment }) });
        if (!response.ok) throw new Error("Quote unavailable");
        setQuote(await response.json());
      }
    } catch { setStatus("That option is unavailable. Try another fulfillment choice."); }
    finally { setBusy(false); }
  };

  const complete = () => onProtectedAction(async () => {
    if (!quote) { await getQuote(); return; }
    setBusy(true);
    try {
      if (STATIC_DEMO) hubStaticDemo.createOrder(listing.id, units, fulfillment);
      else {
        const response = await fetch(`${API}/v1/demo/orders`, { method:"POST", headers:{ "content-type":"application/json" }, body:JSON.stringify({ listingId:listing.id, units, fulfillment }) });
        if (!response.ok) throw new Error("Order unavailable");
      }
      setStatus(`${actionLabel} created in the fictional demo. BSR would hold payment until the transaction is complete.`);
    } catch { setStatus("The demo could not create this transaction."); }
    finally { setBusy(false); }
  });

  return <aside id="booking-card" className="rounded-[24px] bg-white p-6 shadow-soft lg:sticky lg:top-28">
    <p className="text-xs font-bold uppercase tracking-[.14em] text-brand">{listing.listingType === "sale" ? "One-time price" : listing.listingType === "workspace" ? "Space rate" : "Rental rate"}</p>
    <p className="mt-2 font-[Manrope] text-3xl font-bold text-brand">{listingPriceLabel(listing)}</p>
    <p className="mt-2 text-xs text-zinc-500">Transparent quote · 6% BSR service fee</p>
    <div className="mt-6 space-y-4">
      <label className="block text-xs font-bold text-zinc-800">Start date<input type="date" className="mt-2 w-full rounded-xl bg-zinc-50 px-4 py-3 font-normal text-zinc-700 outline-none ring-1 ring-zinc-100 focus:ring-2 focus:ring-brand"/></label>
      <label className="block text-xs font-bold text-zinc-800">{listing.listingType === "sale" ? "Quantity" : listing.billingUnit === "thirty_minutes" ? "30-minute units" : "Rental days"}<input type="number" min="1" value={units} onChange={(event) => { setUnits(Math.max(1, Number(event.target.value))); setQuote(null); }} className="mt-2 w-full rounded-xl bg-zinc-50 px-4 py-3 font-normal text-zinc-700 outline-none ring-1 ring-zinc-100 focus:ring-2 focus:ring-brand"/></label>
      <fieldset><legend className="text-xs font-bold text-zinc-800">Fulfillment</legend><div className="mt-2 grid gap-2">{listing.fulfillment.map((option) => <label key={option} className={`flex cursor-pointer items-center gap-3 rounded-xl p-3 text-xs font-semibold ring-1 transition ${fulfillment === option ? "bg-violet-50 text-brand ring-brand" : "bg-zinc-50 text-zinc-700 ring-zinc-100 hover:bg-white"}`}><input type="radio" name="fulfillment" value={option} checked={fulfillment === option} onChange={() => { setFulfillment(option); setQuote(null); }} className="accent-violet-600"/>{fulfillmentLabel[option]}</label>)}</div></fieldset>
    </div>
    {!quote ? <button disabled={busy} onClick={getQuote} className="mt-6 w-full rounded-full bg-gradient-to-r from-brand to-violet-500 px-6 py-4 text-sm font-bold text-white shadow-lg shadow-violet-500/20 transition hover:-translate-y-0.5 disabled:opacity-50">{busy ? "Calculating…" : "Get protected quote"}</button> : <div className="mt-6 rounded-card bg-violet-50 p-4"><h3 className="text-sm font-bold text-zinc-950">Protected quote</h3><dl className="mt-3 space-y-2 text-xs text-zinc-600"><div className="flex justify-between"><dt>Price</dt><dd>{money(quote.baseCents)}</dd></div><div className="flex justify-between"><dt>BSR fee</dt><dd>{money(quote.serviceFeeCents)}</dd></div><div className="flex justify-between"><dt>Delivery</dt><dd>{money(quote.deliveryFeeCents)}</dd></div><div className="flex justify-between"><dt>Refundable deposit</dt><dd>{money(quote.depositCents)}</dd></div><div className="flex justify-between border-t border-violet-200 pt-3 text-sm font-bold text-zinc-950"><dt>Total held</dt><dd>{money(quote.totalCents)}</dd></div></dl><button disabled={busy} onClick={complete} className="mt-4 w-full rounded-full bg-gradient-to-r from-brand to-violet-500 px-6 py-3.5 text-sm font-bold text-white transition hover:-translate-y-0.5 disabled:opacity-50">{busy ? "Working…" : actionLabel}</button></div>}
    <div className="mt-5 flex flex-wrap gap-2"><span className="inline-flex items-center gap-1 rounded-full bg-violet-50 px-3 py-1.5 text-[10px] font-bold text-brand"><LinearIcon name="shield" className="size-3"/>Protected payment</span><span className="inline-flex items-center gap-1 rounded-full bg-violet-50 px-3 py-1.5 text-[10px] font-bold text-brand"><LinearIcon name="user" className="size-3"/>Verified seller</span>{listing.fulfillment.includes("delivery") && <span className="inline-flex items-center gap-1 rounded-full bg-violet-50 px-3 py-1.5 text-[10px] font-bold text-brand"><LinearIcon name="truck" className="size-3"/>Local delivery</span>}</div>
    {status && <p role="status" className="mt-4 rounded-xl bg-lime-100 p-3 text-xs leading-5 text-zinc-800">{status}</p>}
  </aside>;
}
