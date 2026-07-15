export type AssistantActionId = "rent" | "list" | "workspace" | "delivery" | "payment" | "worker";

export interface AssistantAction {
  id: AssistantActionId;
  label: string;
}

export const assistantActions: AssistantAction[] = [
  { id:"rent", label:"Rent an item" },
  { id:"list", label:"List my item" },
  { id:"workspace", label:"Find a workspace" },
  { id:"delivery", label:"Delivery options" },
  { id:"payment", label:"Payment & deposit" },
  { id:"worker", label:"Talk to a worker" },
];

const responses: Record<AssistantActionId, string> = {
  rent:"I can show rental products nearby. Open a listing to compare the recommended price, deposit, and available pickup or delivery options.",
  list:"Any BSR Hub user can list an item, second-hand product, or workspace. Add its condition, location, and price; the demo keeps exact addresses private.",
  workspace:"I can take you to studios and small production spaces. Spaces that cannot move are booked for on-site use in blocks of at least 30 minutes.",
  delivery:"Delivery availability appears inside each listing. The owner can offer pickup, owner-location use, on-site use, or local Runner delivery.",
  payment:"BSR Hub uses protected payment: funds are held until the transaction is complete. Deposits are shown before reservation and are refundable under the agreed rules.",
  worker:"Write a short description for the BSR team. Do not share payment card details, identity documents, passwords, or an exact home address in this chat.",
};

export function responseForAction(id: AssistantActionId): string {
  return responses[id];
}

export type WorkerHandoff =
  | { mode:"email"; href:string; message:string }
  | { mode:"copy"; message:string };

export function buildWorkerHandoff(message: string, supportDestination: string): WorkerHandoff {
  const cleanMessage = message.trim();
  const destination = supportDestination.trim();
  if (!destination) return { mode:"copy", message:cleanMessage };
  const subject = encodeURIComponent("BSR Hub customer help request");
  const body = encodeURIComponent(cleanMessage);
  return { mode:"email", href:`mailto:${destination}?subject=${subject}&body=${body}`, message:cleanMessage };
}
