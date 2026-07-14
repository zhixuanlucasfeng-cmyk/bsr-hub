# BSR Runner state and role matrix

The Rust backend is authoritative. The frontend may display suggested actions, but it cannot directly change a task row or release money.

| Current state | Role | Action | Next state | Protected effect |
|---|---|---|---|---|
| `draft` | customer | quote | `quoted` | Save integer-cent price explanation |
| `quoted` | customer | fund | `funded` | Create provider payment in `held` state |
| `funded` | customer | publish | `available` | Show only general pickup/drop-off areas |
| `available` | approved runner | accept | `accepted` | Assign exactly one runner; reveal private route to them |
| `accepted` | assigned runner | confirm pickup | `picked_up` | Append immutable event |
| `picked_up` | assigned runner | start delivery | `delivering` | Append immutable event |
| `delivering` | customer | complete + valid code | `completed` | Consume code and release held payout once |
| active state | participant | dispute | `disputed` | Freeze payout for human review |
| pre-pickup state | customer | cancel | `cancelled` | Refund according to policy |
| `available` | server/admin | expire | `expired` | Remove stale job and refund hold |

## Explicitly forbidden transitions

- A runner cannot accept a task unless their profile is approved.
- A different runner cannot update an accepted task.
- A customer cannot mark pickup or delivery started.
- A runner cannot complete their own delivery or release their own payout.
- A completion code cannot be read from the client database API.
- A completed, cancelled, expired or disputed task cannot return to an active state.
- Payment release is idempotent: repeated completion requests cannot pay twice.

## Privacy boundary

`runner_tasks` contains marketplace-safe areas. Exact addresses live in `runner_task_private_locations`; RLS limits them to the customer and assigned runner. Payment mutations, event writes, application approvals and completion secrets have no authenticated client-write policy and must pass through the Rust service.
