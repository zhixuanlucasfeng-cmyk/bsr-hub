export type RunnerTaskState =
  | "draft"
  | "quoted"
  | "funded"
  | "available"
  | "accepted"
  | "picked_up"
  | "delivering"
  | "completed"
  | "cancelled"
  | "expired"
  | "disputed";

export type RunnerPersona = "customer" | "runner" | "admin";
export type RunnerAction =
  | "quote"
  | "fund"
  | "publish"
  | "accept"
  | "confirm_pickup"
  | "start_delivery"
  | "complete"
  | "cancel"
  | "dispute"
  | "expire";

export type TaskCategory =
  | "bsr_rental_delivery"
  | "bsr_second_hand_delivery"
  | "package_pickup"
  | "grocery_pickup"
  | "document_delivery"
  | "small_item_delivery"
  | "other_errand"
  | "prohibited"
  | "medical_emergency";

export interface RunnerQuoteInput {
  category: TaskCategory;
  distance_tenths_mile: number;
  estimated_minutes: number;
  weight: "light" | "medium" | "heavy";
  urgency: "flexible" | "same_day" | "immediate";
  waiting_minutes: number;
}

export interface RunnerQuote {
  runner_payout_cents: number;
  service_fee_cents: number;
  total_cents: number;
  currency: "usd";
  explanation: string[];
}

export interface RunnerTask extends RunnerQuote {
  id: string;
  title: string;
  description: string;
  category: TaskCategory;
  pickup_area: string;
  dropoff_area: string;
  distance_tenths_mile: number;
  estimated_minutes: number;
  weight: "light" | "medium" | "heavy";
  urgency: "flexible" | "same_day" | "immediate";
  state: RunnerTaskState;
  assigned_runner_id?: string | null;
  pickup_address?: string;
  dropoff_address?: string;
}

export interface RunnerApplication {
  id: string;
  name: string;
  age_confirmed: boolean;
  transport: string;
  service_radius_miles: number;
  status: "pending" | "approved" | "rejected";
}

export interface RunnerEarnings {
  runner_id: string;
  available_cents: number;
  completed_tasks: number;
  currency: "usd";
}

export interface AdminSummary {
  total_tasks: number;
  active_tasks: number;
  pending_applications: number;
  approved_runners: number;
  completed_tasks: number;
  disputed_tasks: number;
  prohibited_tasks_blocked: number;
}

export const money = (cents: number): string =>
  new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  }).format(cents / 100);

export function actionsFor(state: RunnerTaskState, persona: RunnerPersona): RunnerAction[] {
  if (persona === "customer") {
    const customerActions: Partial<Record<RunnerTaskState, RunnerAction[]>> = {
      draft: ["quote", "cancel"],
      quoted: ["fund", "cancel"],
      funded: ["publish", "cancel"],
      available: ["cancel"],
      accepted: ["dispute"],
      picked_up: ["dispute"],
      delivering: ["complete", "dispute"],
    };
    return customerActions[state] ?? [];
  }
  if (persona === "runner") {
    const runnerActions: Partial<Record<RunnerTaskState, RunnerAction[]>> = {
      available: ["accept"],
      accepted: ["confirm_pickup", "dispute"],
      picked_up: ["start_delivery", "dispute"],
      delivering: ["dispute"],
    };
    return runnerActions[state] ?? [];
  }
  return state === "available" ? ["expire"] : [];
}

const progress: Record<RunnerTaskState, number> = {
  draft: 5,
  quoted: 10,
  funded: 18,
  available: 25,
  accepted: 45,
  picked_up: 70,
  delivering: 85,
  completed: 100,
  cancelled: 100,
  expired: 100,
  disputed: 100,
};

export const taskProgress = (state: RunnerTaskState): number => progress[state];

export const actionLabel: Record<RunnerAction, string> = {
  quote: "Get automatic quote",
  fund: "Fund protected payment",
  publish: "Publish to runners",
  accept: "Accept this job",
  confirm_pickup: "Confirm pickup",
  start_delivery: "Start delivery",
  complete: "Enter completion code",
  cancel: "Cancel task",
  dispute: "Report a problem",
  expire: "Expire task",
};
