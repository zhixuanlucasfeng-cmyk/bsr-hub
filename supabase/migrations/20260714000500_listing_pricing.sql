create table if not exists public.listing_pricing_profiles (
  listing_id uuid primary key references public.listings(id) on delete cascade,
  category text not null check (category in ('ps5', 'workspace')),
  billing_unit text not null check (billing_unit in ('thirty_minutes', 'day')),
  attributes jsonb not null,
  ruleset_version text not null check (ruleset_version = 'rules-v1'),
  recommended_unit_price_cents bigint not null check (recommended_unit_price_cents >= 0),
  seller_adjustment_cents bigint not null check (seller_adjustment_cents between -500 and 500),
  final_unit_price_cents bigint generated always as
    (recommended_unit_price_cents + seller_adjustment_cents) stored,
  allowed_fulfillment_methods text[] not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (recommended_unit_price_cents + seller_adjustment_cents >= 0),
  check (cardinality(allowed_fulfillment_methods) > 0),
  check (
    allowed_fulfillment_methods <@
    array['pickup', 'delivery', 'owner_location', 'on_site']::text[]
  )
);

alter table public.listing_pricing_profiles enable row level security;
revoke insert, update, delete on public.listing_pricing_profiles from anon, authenticated;
