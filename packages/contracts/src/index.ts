export type FulfillmentMethod =
  | "pickup"
  | "delivery"
  | "owner_location"
  | "on_site";

export type BillingUnit = "thirty_minutes" | "day";
export type Ps5Model = "original" | "slim" | "pro";
export type Condition = "like_new" | "good" | "fair" | "worn";
export type LocationTier = "residential" | "suburban" | "urban" | "premium";

export interface Ps5PricingAttributes {
  category: "ps5";
  model: Ps5Model;
  ageMonths: number;
  condition: Condition;
  cleanliness: number;
  fullyOperational: boolean;
  missingNonessentialFeatures: number;
  controllerCount: number;
  billingUnit: BillingUnit;
}

export interface WorkspacePricingAttributes {
  category: "workspace";
  squareFeet: number;
  locationTier: LocationTier;
  cleanliness: number;
  equipmentScore: number;
  amenityCount: number;
  billingUnit: BillingUnit;
}

export type PricingAttributes = Ps5PricingAttributes | WorkspacePricingAttributes;

export interface SavePricingRequest {
  attributes: PricingAttributes;
  sellerAdjustmentCents: number;
  allowedFulfillmentMethods: FulfillmentMethod[];
}

export interface PricingProfile {
  listingId: string;
  recommendedUnitPriceCents: number;
  sellerAdjustmentCents: number;
  finalUnitPriceCents: number;
  minimumAllowedCents: number;
  maximumAllowedCents: number;
  billingUnit: BillingUnit;
  rulesetVersion: string;
  reasonCodes: string[];
  allowedFulfillmentMethods: FulfillmentMethod[];
}

export interface QuoteRequest {
  listingId: string;
  startAt: string;
  endAt: string;
  fulfillment: FulfillmentMethod;
}

export interface QuoteBreakdown {
  unitPriceCents: number;
  billableUnits: number;
  billingUnit: BillingUnit;
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
  startAt: string;
  endAt: string;
  fulfillment: FulfillmentMethod;
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
