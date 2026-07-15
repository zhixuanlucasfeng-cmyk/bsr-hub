export type Fulfillment = "pickup" | "delivery" | "owner_location" | "on_site";
export type OrderState = "pending_payment" | "paid" | "confirmed" | "active" | "fulfilled" | "returned" | "completed" | "cancelled" | "expired";

export interface Listing {
  id: string; ownerId: string; title: string; listingType: "rental" | "sale" | "workspace";
  category: string; description: string; city: string; state: string; condition: string;
  unitPriceCents: number; depositCents: number; deliveryFeeCents: number;
  billingUnit: "thirty_minutes" | "day"; fulfillment: Fulfillment[]; accent: string; icon: string;
  imageSrc: string; imageAlt: string;
}

export interface Quote {
  unitPriceCents: number; billableUnits: number; billingUnit: string; baseCents: number;
  serviceFeeCents: number; deliveryFeeCents: number; depositCents: number; totalCents: number; currency: string;
}

export interface DemoOrder {
  id: string; listingId: string; listingTitle: string; buyerId: string; sellerId: string;
  state: OrderState; fulfillment: Fulfillment; quote: Quote;
}

export const personas = [
  { id: "buyer-demo", name: "Maya", role: "Buyer", initials: "MB" },
  { id: "seller-demo", name: "Jordan", role: "Seller", initials: "JS" },
  { id: "business-demo", name: "Alex", role: "Small business", initials: "AB" }
] as const;

export function money(cents: number) { return new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" }).format(cents / 100); }

export function allowedActions(state: OrderState, persona: string, order: DemoOrder) {
  if (state === "pending_payment" && persona === order.buyerId) return ["mark_paid", "cancel"];
  if (state === "paid" && persona === order.sellerId) return ["confirm", "cancel"];
  if (state === "confirmed" && persona === order.sellerId) return ["activate", "fulfill", "cancel"];
  if (state === "active" && persona === order.buyerId) return ["return"];
  if ((state === "returned" || state === "fulfilled") && persona === order.sellerId) return ["complete"];
  return [];
}
