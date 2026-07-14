-- BSR Runner marketplace. Monetary values are integer cents and private route data
-- is separated so public task discovery never exposes a home or business address.
create table if not exists runner_profiles (
  profile_id uuid primary key references profiles(id) on delete cascade,
  status text not null default 'pending' check (status in ('pending','approved','rejected','suspended')),
  transport text not null check (transport in ('walking','bike','scooter','car')),
  service_radius_miles smallint not null check (service_radius_miles between 1 and 100),
  age_confirmed boolean not null default false,
  approved_at timestamptz,
  created_at timestamptz not null default now()
);

create table if not exists runner_tasks (
  id uuid primary key default gen_random_uuid(),
  customer_id uuid not null references profiles(id),
  assigned_runner_id uuid references runner_profiles(profile_id),
  category text not null check (category in (
    'bsr_rental_delivery','bsr_second_hand_delivery','package_pickup','grocery_pickup',
    'document_delivery','small_item_delivery','other_errand'
  )),
  title text not null check (char_length(title) between 3 and 120),
  description text not null check (char_length(description) between 3 and 1500),
  pickup_area text not null,
  dropoff_area text not null,
  distance_tenths_mile integer not null check (distance_tenths_mile between 1 and 1000),
  estimated_minutes integer not null check (estimated_minutes between 5 and 480),
  waiting_minutes integer not null default 0 check (waiting_minutes between 0 and 120),
  weight text not null check (weight in ('light','medium','heavy')),
  urgency text not null check (urgency in ('flexible','same_day','immediate')),
  state text not null default 'draft' check (state in (
    'draft','quoted','funded','available','accepted','picked_up','delivering',
    'completed','cancelled','expired','disputed'
  )),
  runner_payout_cents bigint not null check (runner_payout_cents >= 0),
  service_fee_cents bigint not null check (service_fee_cents >= 0),
  total_cents bigint not null check (total_cents = runner_payout_cents + service_fee_cents),
  currency text not null default 'USD' check (currency = 'USD'),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists runner_task_private_locations (
  task_id uuid primary key references runner_tasks(id) on delete cascade,
  pickup_address text not null,
  dropoff_address text not null,
  pickup_instructions text not null default '',
  dropoff_instructions text not null default ''
);

create table if not exists runner_task_payments (
  task_id uuid primary key references runner_tasks(id) on delete cascade,
  provider text not null check (provider in ('stripe','demo')),
  provider_reference text not null unique,
  status text not null check (status in ('pending','held','released','refunded','failed')),
  amount_cents bigint not null check (amount_cents >= 0),
  currency text not null default 'USD' check (currency = 'USD'),
  released_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists runner_task_events (
  id bigint generated always as identity primary key,
  task_id uuid not null references runner_tasks(id) on delete cascade,
  actor_id uuid references profiles(id),
  from_state text,
  to_state text not null,
  action text not null,
  metadata jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);

create table if not exists runner_completion_secrets (
  task_id uuid primary key references runner_tasks(id) on delete cascade,
  code_hash text not null,
  failed_attempts smallint not null default 0 check (failed_attempts between 0 and 10),
  consumed_at timestamptz
);

create index if not exists runner_tasks_market_idx on runner_tasks(state, created_at desc)
  where state = 'available';
create index if not exists runner_tasks_customer_idx on runner_tasks(customer_id, created_at desc);
create index if not exists runner_tasks_assignee_idx on runner_tasks(assigned_runner_id, created_at desc);
create index if not exists runner_task_events_task_idx on runner_task_events(task_id, created_at);

alter table runner_profiles enable row level security;
alter table runner_tasks enable row level security;
alter table runner_task_private_locations enable row level security;
alter table runner_task_payments enable row level security;
alter table runner_task_events enable row level security;
alter table runner_completion_secrets enable row level security;

create policy "people read their runner profile" on runner_profiles for select
  using (profile_id = auth.uid());
create policy "people submit their runner profile" on runner_profiles for insert
  with check (profile_id = auth.uid() and age_confirmed);

create policy "available runner tasks are discoverable" on runner_tasks for select
  using (state = 'available');
create policy "task participants read task" on runner_tasks for select
  using (customer_id = auth.uid() or assigned_runner_id = auth.uid());
create policy "customers create draft tasks" on runner_tasks for insert
  with check (customer_id = auth.uid() and state = 'draft' and assigned_runner_id is null);

-- An exact address appears only to its customer or the assigned approved runner.
create policy "task participants read private route" on runner_task_private_locations for select
  using (exists (
    select 1 from runner_tasks task
    where task.id = task_id
      and (task.customer_id = auth.uid() or task.assigned_runner_id = auth.uid())
  ));
create policy "customers add private route" on runner_task_private_locations for insert
  with check (exists (
    select 1 from runner_tasks task
    where task.id = task_id and task.customer_id = auth.uid() and task.state = 'draft'
  ));

create policy "task participants read protected payment" on runner_task_payments for select
  using (exists (
    select 1 from runner_tasks task
    where task.id = task_id
      and (task.customer_id = auth.uid() or task.assigned_runner_id = auth.uid())
  ));
create policy "task participants read event log" on runner_task_events for select
  using (exists (
    select 1 from runner_tasks task
    where task.id = task_id
      and (task.customer_id = auth.uid() or task.assigned_runner_id = auth.uid())
  ));

-- Intentionally no client UPDATE policies for tasks, runner approvals, payments,
-- events or completion secrets. The Rust API validates every state transition and
-- uses the service role for these writes. Completion codes are never client-readable.
