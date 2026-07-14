export type FulfillmentMethod =
  | "pickup"
  | "delivery"
  | "owner_location"
  | "on_site";

export interface QuoteRequest {
  listingId: string;
  units: number;
  wantsDelivery: boolean;
  startAt: string | null;
  endAt: string | null;
  fulfillment: FulfillmentMethod;
}

export interface QuoteBreakdown {
  baseCents: number;
  serviceFeeCents: number;
  deliveryFeeCents: number;
  depositCents: number;
  totalCents: number;
  currency: "USD";
}

export interface ApiError {
  error: { code: string; message: string; requestId: string };
}

export interface CreateOrderRequest {
  listingId: string;
  units: number;
  wantsDelivery: boolean;
  startAt?: string | null;
  endAt?: string | null;
}

export interface CreateOrderResponse {
  orderId: string;
  reservationExpiresAt: string;
  totalCents: number;
  checkoutUrl: string;
}

export type OrderState =
  | "pending_payment"
  | "paid"
  | "confirmed"
  | "active"
  | "fulfilled"
  | "returned"
  | "completed"
  | "cancelled"
  | "expired";

export type OrderAction =
  | "confirm"
  | "activate"
  | "fulfill"
  | "return"
  | "complete"
  | "cancel";
