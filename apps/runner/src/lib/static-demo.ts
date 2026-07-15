import type { AdminSummary, RunnerAction, RunnerApplication, RunnerEarnings, RunnerPersona, RunnerQuote, RunnerQuoteInput, RunnerTask, RunnerTaskState, TaskCategory } from "./contracts.ts";

type StoredTask = RunnerTask & { pickup_address:string; dropoff_address:string; completion_code:string; payout_released:boolean };
type CreateTask = RunnerQuoteInput & Pick<StoredTask, "title"|"description"|"pickup_area"|"dropoff_area"|"pickup_address"|"dropoff_address"> & { customer_id:string; safety_confirmed:boolean };
type ActionInput = { action:RunnerAction; role:RunnerPersona; runner_id?:string; completion_code?:string };

const categorySurcharge: Record<TaskCategory, number> = { bsr_rental_delivery:150, bsr_second_hand_delivery:150, package_pickup:100, grocery_pickup:250, document_delivery:100, small_item_delivery:100, other_errand:300, prohibited:0, medical_emergency:0 };
const categoryLabel: Record<TaskCategory, string> = { bsr_rental_delivery:"BSR rental delivery", bsr_second_hand_delivery:"BSR second-hand delivery", package_pickup:"package pickup", grocery_pickup:"grocery pickup", document_delivery:"document delivery", small_item_delivery:"small-item delivery", other_errand:"local errand", prohibited:"prohibited task", medical_emergency:"medical emergency" };
const weightSurcharge = { light:0, medium:250, heavy:600 } as const;
const urgencySurcharge = { flexible:0, same_day:200, immediate:800 } as const;
const urgencyLabel = { flexible:"flexible", same_day:"same-day", immediate:"immediate" } as const;

export function quoteRunnerTask(input: RunnerQuoteInput): RunnerQuote {
  if (input.category === "prohibited") throw new Error("This task type is prohibited");
  if (input.category === "medical_emergency") throw new Error("Contact emergency services instead of creating this task");
  if (input.distance_tenths_mile < 1 || input.distance_tenths_mile > 1000 || input.estimated_minutes < 5 || input.estimated_minutes > 480 || input.waiting_minutes < 0 || input.waiting_minutes > 120) throw new Error("Distance, time, or waiting input is outside the supported range");
  const runner_payout_cents = Math.max(1200, 650 + input.distance_tenths_mile * 19 + input.estimated_minutes * 35 + input.waiting_minutes * 30 + categorySurcharge[input.category] + weightSurcharge[input.weight] + urgencySurcharge[input.urgency]);
  const service_fee_cents = Math.trunc((runner_payout_cents * 12 + 99) / 100);
  return { runner_payout_cents, service_fee_cents, total_cents:runner_payout_cents + service_fee_cents, currency:"usd", explanation:[`Base pay for ${categoryLabel[input.category]}`, `Distance allowance for ${(input.distance_tenths_mile / 10).toFixed(1)} miles`, `Time allowance for ${input.estimated_minutes} minutes`, `${input.weight} item · ${urgencyLabel[input.urgency]} timing`, "BSR Runner service fee: 12%"] };
}

function seedTask(id:string, title:string, description:string, category:TaskCategory, pickup_area:string, dropoff_area:string, distance_tenths_mile:number, estimated_minutes:number, weight:RunnerQuoteInput["weight"], urgency:RunnerQuoteInput["urgency"], completion_code:string):StoredTask {
  const quote = quoteRunnerTask({ category, distance_tenths_mile, estimated_minutes, weight, urgency, waiting_minutes:0 });
  return { id, title, description, category, pickup_area, dropoff_area, pickup_address:`${id.slice(5)} Fictional Pickup`, dropoff_address:`${id.slice(5)} Demo Dropoff`, distance_tenths_mile, estimated_minutes, weight, urgency, state:"available", assigned_runner_id:null, ...quote, completion_code, payout_released:false };
}

function seededTasks(): StoredTask[] { return [
  seedTask("task-1", "Pick up a package and deliver it nearby", "A sealed medium package; no stairs or residence entry.", "package_pickup", "Wellesley Square", "Babson Park", 32, 35, "medium", "same_day", "482731"),
  seedTask("task-2", "Deliver a rented camera kit", "BSR Hub protected rental delivery in a padded case.", "bsr_rental_delivery", "Needham Center", "Newton Highlands", 58, 48, "medium", "flexible", "735204"),
  seedTask("task-3", "Pick up groceries for a student team", "Four prepaid grocery bags from the service counter.", "grocery_pickup", "Wellesley Hills", "Babson Park", 27, 32, "heavy", "immediate", "195628"),
  seedTask("task-4", "Deliver signed event documents", "One sealed folder for a campus event organizer.", "document_delivery", "Weston Center", "Wellesley Square", 71, 52, "light", "same_day", "604319"),
]; }

const nextState: Partial<Record<RunnerTaskState, Partial<Record<RunnerPersona, Partial<Record<RunnerAction,RunnerTaskState>>>>>> = {
  draft:{ customer:{ quote:"quoted", cancel:"cancelled" } }, quoted:{ customer:{ fund:"funded", cancel:"cancelled" } }, funded:{ customer:{ publish:"available", cancel:"cancelled" } },
  available:{ customer:{ cancel:"cancelled" }, runner:{ accept:"accepted" }, admin:{ expire:"expired" } },
  accepted:{ customer:{ dispute:"disputed" }, runner:{ confirm_pickup:"picked_up", dispute:"disputed" }, admin:{ dispute:"disputed" } },
  picked_up:{ customer:{ dispute:"disputed" }, runner:{ start_delivery:"delivering", dispute:"disputed" }, admin:{ dispute:"disputed" } },
  delivering:{ customer:{ complete:"completed", dispute:"disputed" }, runner:{ dispute:"disputed" }, admin:{ dispute:"disputed" } },
};

function body<T>(init?:RequestInit):T { return init?.body ? JSON.parse(String(init.body)) as T : {} as T; }
function publicTask(task:StoredTask, reveal=false):RunnerTask { const { completion_code:_code, payout_released:_released, pickup_address, dropoff_address, ...value } = task; return reveal ? { ...value, pickup_address, dropoff_address } : value; }

export function createRunnerStaticDemo() {
  let tasks = seededTasks();
  let applications:RunnerApplication[] = [{ id:"application-1", name:"Jordan Smith", age_confirmed:true, transport:"car", service_radius_miles:12, status:"approved" }];
  let earnings:RunnerEarnings = { runner_id:"runner-1", available_cents:0, completed_tasks:0, currency:"usd" };
  let prohibitedBlocked = 3;

  async function request<T=unknown>(rawPath:string, init?:RequestInit):Promise<T> {
    const url = new URL(rawPath, "https://demo.local"); const path = url.pathname; const method = init?.method ?? "GET";
    if (path === "/v1/runner/demo/quote" && method === "POST") { const input=body<RunnerQuoteInput>(init); try{return quoteRunnerTask(input) as T;}catch(error){if(input.category==="prohibited")prohibitedBlocked++;throw error;} }
    if (path === "/v1/runner/demo/tasks" && method === "GET") return tasks.map((task)=>publicTask(task)) as T;
    if (path === "/v1/runner/demo/tasks" && method === "POST") {
      const input=body<CreateTask>(init); if(!input.safety_confirmed || !input.title?.trim() || !input.description?.trim() || !input.pickup_area?.trim() || !input.dropoff_area?.trim() || !input.pickup_address?.trim() || !input.dropoff_address?.trim()) throw new Error("Task details and the safety confirmation are required");
      const quote=quoteRunnerTask(input); const number=tasks.length+1; const task:StoredTask={ id:`task-${number}`, title:input.title.trim(), description:input.description.trim(), category:input.category, pickup_area:input.pickup_area.trim(), dropoff_area:input.dropoff_area.trim(), pickup_address:input.pickup_address.trim(), dropoff_address:input.dropoff_address.trim(), distance_tenths_mile:input.distance_tenths_mile, estimated_minutes:input.estimated_minutes, weight:input.weight, urgency:input.urgency, state:"quoted", assigned_runner_id:null, ...quote, completion_code:String(410000+tasks.length).padStart(6,"0"), payout_released:false }; tasks.push(task); return publicTask(task) as T;
    }
    const detail=path.match(/^\/v1\/runner\/demo\/tasks\/([^/]+)$/); if(detail && method==="GET") { const task=tasks.find((item)=>item.id===detail[1]); if(!task)throw new Error("Task not found"); const reveal=Boolean(task.assigned_runner_id && task.assigned_runner_id===url.searchParams.get("runner_id")); return publicTask(task,reveal) as T; }
    const actionPath=path.match(/^\/v1\/runner\/demo\/tasks\/([^/]+)\/actions$/); if(actionPath && method==="POST") {
      const input=body<ActionInput>(init); const task=tasks.find((item)=>item.id===actionPath[1]); if(!task)throw new Error("Task not found");
      if(input.role==="runner" && input.action!=="accept" && task.assigned_runner_id!==input.runner_id)throw new Error("Only the assigned runner can update this task");
      if(input.action==="complete" && input.completion_code!==task.completion_code)throw new Error("The completion code is incorrect");
      const state=nextState[task.state]?.[input.role]?.[input.action]; if(!state)throw new Error("Action is not allowed"); task.state=state;
      if(input.action==="accept"){if(!input.runner_id)throw new Error("A runner ID is required");task.assigned_runner_id=input.runner_id;}
      if(state==="completed"&&!task.payout_released){task.payout_released=true;earnings={...earnings,available_cents:earnings.available_cents+task.runner_payout_cents,completed_tasks:earnings.completed_tasks+1};}
      return publicTask(task) as T;
    }
    if(path === "/v1/runner/demo/applications" && method === "GET") return applications.map((value)=>({...value})) as T;
    if(path === "/v1/runner/demo/applications" && method === "POST") { const input=body<Omit<RunnerApplication,"id"|"status">>(init); if(!input.age_confirmed||!input.name?.trim()||input.service_radius_miles<=0)throw new Error("Applicants must confirm age 18+ and provide a service area"); const application:RunnerApplication={...input,name:input.name.trim(),id:`application-${applications.length+1}`,status:"pending"};applications.push(application);return {...application} as T; }
    const approval=path.match(/^\/v1\/runner\/demo\/applications\/([^/]+)\/approve$/); if(approval&&method==="POST"){const application=applications.find((item)=>item.id===approval[1]);if(!application)throw new Error("Application not found");application.status="approved";return {...application} as T;}
    if(path === "/v1/runner/demo/earnings/runner-1" && method === "GET") return {...earnings} as T;
    if(path === "/v1/runner/demo/admin" && method === "GET") { const activeStates=new Set(["available","accepted","picked_up","delivering"]); const summary:AdminSummary={total_tasks:tasks.length,active_tasks:tasks.filter((task)=>activeStates.has(task.state)).length,pending_applications:applications.filter((item)=>item.status==="pending").length,approved_runners:applications.filter((item)=>item.status==="approved").length,completed_tasks:tasks.filter((task)=>task.state==="completed").length,disputed_tasks:tasks.filter((task)=>task.state==="disputed").length,prohibited_tasks_blocked:prohibitedBlocked};return summary as T; }
    if(path === "/v1/runner/demo/reset" && method === "POST") { tasks=seededTasks();applications=[{id:"application-1",name:"Jordan Smith",age_confirmed:true,transport:"car",service_radius_miles:12,status:"approved"}];earnings={runner_id:"runner-1",available_cents:0,completed_tasks:0,currency:"usd"};prohibitedBlocked=3;return undefined as T; }
    throw new Error(`Static demo route not found: ${method} ${path}`);
  }
  return { request };
}
