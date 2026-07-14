create table if not exists listing_images (
  id uuid primary key default gen_random_uuid(),
  listing_id uuid not null references listings(id) on delete cascade,
  storage_path text not null,
  alt_text text not null,
  position integer not null default 0 check (position between 0 and 9),
  unique (listing_id, position)
);

create table if not exists listing_availability (
  id uuid primary key default gen_random_uuid(),
  listing_id uuid not null references listings(id) on delete cascade,
  starts_at timestamptz not null,
  ends_at timestamptz not null,
  check (starts_at < ends_at)
);

create table if not exists payments (
  order_id uuid primary key references orders(id) on delete cascade,
  provider text not null check (provider in ('stripe', 'demo')),
  provider_reference text not null unique,
  status text not null check (status in ('pending','held','released','refunded','failed')),
  amount_cents bigint not null check (amount_cents >= 0),
  currency text not null default 'USD' check (currency = 'USD'),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists reviews (
  id uuid primary key default gen_random_uuid(),
  order_id uuid not null unique references orders(id) on delete cascade,
  reviewer_id uuid not null references profiles(id),
  reviewee_id uuid not null references profiles(id),
  rating smallint not null check (rating between 1 and 5),
  comment text not null default '' check (char_length(comment) <= 1000),
  created_at timestamptz not null default now(),
  check (reviewer_id <> reviewee_id)
);

alter table listing_images enable row level security;
alter table listing_availability enable row level security;
alter table payments enable row level security;
alter table reviews enable row level security;

create policy "active listing images are readable" on listing_images for select using (
  exists (select 1 from listings where listings.id = listing_id and listings.status = 'active')
);
create policy "owners manage listing images" on listing_images for all using (
  auth.uid() = (select owner_id from listings where id = listing_id)
) with check (
  auth.uid() = (select owner_id from listings where id = listing_id)
);

create policy "active availability is readable" on listing_availability for select using (
  exists (select 1 from listings where listings.id = listing_id and listings.status = 'active')
);
create policy "owners manage availability" on listing_availability for all using (
  auth.uid() = (select owner_id from listings where id = listing_id)
) with check (
  auth.uid() = (select owner_id from listings where id = listing_id)
);

create policy "participants read payment status" on payments for select using (
  exists (
    select 1 from orders join listings on listings.id = orders.listing_id
    where orders.id = order_id and (orders.buyer_id = auth.uid() or listings.owner_id = auth.uid())
  )
);
-- No authenticated INSERT/UPDATE/DELETE policy exists for payments. Only the server's
-- service role can record or release protected funds.

create policy "public completed reviews are readable" on reviews for select using (
  exists (select 1 from orders where orders.id = order_id and orders.status = 'completed')
);
create policy "buyers review completed orders" on reviews for insert with check (
  reviewer_id = auth.uid() and exists (
    select 1 from orders join listings on listings.id = orders.listing_id
    where orders.id = order_id
      and orders.status = 'completed'
      and orders.buyer_id = auth.uid()
      and reviewee_id = listings.owner_id
  )
);

create index if not exists listing_images_listing_idx on listing_images(listing_id, position);
create index if not exists listing_availability_listing_time_idx on listing_availability(listing_id, starts_at, ends_at);
create index if not exists reviews_reviewee_idx on reviews(reviewee_id, created_at desc);
