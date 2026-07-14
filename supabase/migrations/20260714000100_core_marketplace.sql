create extension if not exists btree_gist;

create table if not exists profiles (
  id uuid primary key,
  display_name text not null,
  city text not null,
  state text not null check (char_length(state) = 2),
  created_at timestamptz not null default now()
);

create table if not exists listings (
  id uuid primary key default gen_random_uuid(),
  owner_id uuid not null references profiles(id),
  listing_type text not null check (listing_type in ('rental', 'sale', 'workspace')),
  title text not null,
  unit_price_cents bigint not null check (unit_price_cents >= 0),
  deposit_cents bigint not null default 0 check (deposit_cents >= 0),
  delivery_fee_cents bigint not null default 0 check (delivery_fee_cents >= 0),
  status text not null default 'draft' check (status in ('draft', 'active', 'paused', 'sold', 'archived')),
  city text not null,
  state text not null check (char_length(state) = 2),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists listing_private_locations (
  listing_id uuid primary key references listings(id) on delete cascade,
  street_address text not null,
  unit text,
  postal_code text not null
);

create table if not exists orders (
  id uuid primary key default gen_random_uuid(),
  listing_id uuid not null references listings(id),
  buyer_id uuid not null references profiles(id),
  status text not null check (status in ('pending_payment','paid','confirmed','active','fulfilled','returned','completed','cancelled','expired')),
  start_at timestamptz,
  end_at timestamptz,
  reservation_expires_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check ((start_at is null and end_at is null) or (start_at is not null and end_at is not null and start_at < end_at))
);

alter table orders drop constraint if exists orders_no_overlapping_active_booking;
alter table orders add constraint orders_no_overlapping_active_booking
  exclude using gist (
    listing_id with =,
    tstzrange(start_at, end_at, '[)') with &&
  ) where (status in ('pending_payment','paid','confirmed','active'));

create table if not exists order_amounts (
  order_id uuid primary key references orders(id) on delete cascade,
  base_cents bigint not null check (base_cents >= 0),
  service_fee_cents bigint not null check (service_fee_cents >= 0),
  delivery_fee_cents bigint not null check (delivery_fee_cents >= 0),
  deposit_cents bigint not null check (deposit_cents >= 0),
  total_cents bigint not null check (total_cents >= 0),
  currency text not null default 'USD' check (currency = 'USD')
);

create table if not exists order_events (
  id bigint generated always as identity primary key,
  order_id uuid not null references orders(id) on delete cascade,
  event_type text not null,
  actor_id uuid references profiles(id),
  created_at timestamptz not null default now()
);

create table if not exists stripe_events (
  event_id text primary key,
  processed_at timestamptz not null default now()
);

create index if not exists orders_listing_status_idx on orders(listing_id, status);
create index if not exists listings_public_search_idx on listings(status, state, city);

alter table profiles enable row level security;
alter table listings enable row level security;
alter table listing_private_locations enable row level security;
alter table orders enable row level security;
alter table order_amounts enable row level security;
alter table order_events enable row level security;

create policy "public active listings are readable" on listings
  for select using (status = 'active');
create policy "owners manage listings" on listings
  for all using (auth.uid() = owner_id) with check (auth.uid() = owner_id);
create policy "owners manage private listing locations" on listing_private_locations
  for all using (
    auth.uid() = (select owner_id from listings where id = listing_id)
  ) with check (
    auth.uid() = (select owner_id from listings where id = listing_id)
  );
create policy "participants read orders" on orders
  for select using (
    auth.uid() = buyer_id or auth.uid() = (select owner_id from listings where id = listing_id)
  );
