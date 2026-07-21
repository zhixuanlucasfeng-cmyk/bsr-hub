# Render + MongoDB Atlas production runbook

This runbook deploys the Rust/Axum core API without exposing database credentials to either frontend. The public GitHub Pages demos remain fictional and browser-local until the deployed API, Supabase authentication, and Stripe test mode are connected deliberately.

## Architecture

```text
GitHub Pages (BSR Hub / Runner)
        |
        | HTTPS API requests
        v
Render: bsr-hub-core-api (Rust/Axum)
        |
        | TLS using MONGODB_URI
        v
MongoDB Atlas: bsr_hub
```

Only Render receives `MONGODB_URI`. Never add this value to GitHub, client-side JavaScript, `NEXT_PUBLIC_*`, screenshots, or chat messages.

## 1. Create Atlas resources

1. In MongoDB Atlas, create a free classroom cluster where the free option is available.
2. Create a database user with a long generated password and read/write access only to `bsr_hub`.
3. In Atlas **Network Access**, add the outbound IP ranges shown by the Render service. Do not allow the whole internet (`0.0.0.0/0`) for a real launch.
4. Copy the `mongodb+srv://...` driver connection string, insert the database user's URL-encoded password, and keep it private.

Atlas must run as a replica set because BSR Hub uses MongoDB transactions to protect reservations and payments.

## 2. Create the Render Blueprint

1. In Render, create a new Blueprint from this GitHub repository.
2. Render reads `render.yaml` and creates `bsr-hub-core-api` on the explicitly configured free plan.
3. Enter the prompted secret values:

| Variable | Value |
| --- | --- |
| `MONGODB_URI` | Private Atlas `mongodb+srv://...` URI |
| `SUPABASE_URL` | Development Supabase project URL |
| `SUPABASE_ANON_KEY` | Supabase anon/publishable key |
| `STRIPE_SECRET_KEY` | Stripe **test-mode** key beginning `sk_test_` |
| `STRIPE_WEBHOOK_SECRET` | Stripe test webhook secret beginning `whsec_` |

The Blueprint already supplies `MONGODB_DATABASE`, GitHub Pages callback URLs, the allowed browser origin, the 6% fee, and the 30-minute reservation timeout. It runs `mongo_bootstrap` once on initial deployment to create indexes and fictional classroom listings. Later deploys do not reset inventory.

## 3. Configure Stripe test webhooks

After Render assigns the service URL, create a Stripe test webhook pointing to:

```text
https://<render-service-host>/v1/webhooks/stripe
```

Subscribe only to the Checkout events used by the API and replace `STRIPE_WEBHOOK_SECRET` in Render with that endpoint's test signing secret. Never use a live Stripe key for this classroom MVP.

## 4. Verify deployment

```bash
curl -fsS https://<render-service-host>/health
curl -fsS https://<render-service-host>/ready
```

Expected responses are `{"status":"ok"}` and `{"status":"ready"}`. `/health` proves the process is running; `/ready` also proves MongoDB is reachable. If `/health` succeeds but `/ready` returns 503, inspect Atlas network access, database credentials, and the Render logs.

Run the repository release gate before changing the public frontend API URL:

```bash
npm run check
```

## 5. Frontend cutover

Keep `NEXT_PUBLIC_STATIC_DEMO=true` for the public classroom links until authentication and all protected order journeys pass against Render. At cutover, rebuild the Hub with `NEXT_PUBLIC_API_URL=https://<render-service-host>` and the Runner with its own verified API URL. MongoDB remains reachable only through Rust.

## Cost and limitations

- The repository pins Render to the free plan, so it cannot silently create a paid Starter service. Free services can sleep and have a cold-start delay.
- MongoDB Atlas has a free option in supported configurations; confirm the selected cluster shows `$0` before creation.
- Stripe test mode does not move real money.
- A real launch still requires identity checks, moderation, insurance, dispute procedures, privacy/legal review, observability, backups, and paid production infrastructure.
