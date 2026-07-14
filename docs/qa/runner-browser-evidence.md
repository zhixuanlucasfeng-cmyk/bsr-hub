# BSR Runner browser verification

Verified on 2026-07-15 against the local Next.js site at `http://127.0.0.1:3001` and Rust demo API at `http://127.0.0.1:8080`.

## Complete protected-task journey

| Check | Evidence observed | Result |
|---|---|---|
| Seed data | 4 open fictional tasks loaded from the Rust API | Pass |
| Unsafe request | `prohibited` quote returned “This task type is prohibited” | Pass |
| Automatic quote | 3.2-mile, 35-minute, medium, same-day task quoted $30.33 runner pay + $3.64 service fee = $33.97 | Pass |
| Customer publish | New task moved through quoted → funded → available | Pass |
| Public privacy | Market and pre-acceptance task showed areas but no exact addresses | Pass |
| Runner accept | Task 1 moved available → accepted and revealed only its fictional assigned-runner route | Pass |
| Runner delivery | Assigned runner moved accepted → picked_up → delivering | Pass |
| Customer completion | Code `482731` moved delivering → completed | Pass |
| Protected payout | Runner wallet changed to $30.33 and 1 completed task | Pass |
| Admin visibility | Safety desk showed task totals, approved runner, completed activity and 3 blocked unsafe-task attempts | Pass |

## Responsive and runtime checks

- Phone viewport tested at 390 × 844.
- `innerWidth`, document width and body width were all exactly 390 px: no horizontal overflow.
- Mobile bottom navigation exposed Jobs and the persona-specific action (Post a task, Earnings or Safety desk).
- Browser console after the full journey: 0 errors and 0 warnings.
- All data, addresses, identities and payments used in this flow were fictional classroom demo values.

## Scope note

The in-memory flow proves the product interaction without collecting sensitive data. The Supabase migration and pgTAP tests document the production data boundary, but a deployed U.S. service still needs legal, insurance, payments, tax, background-screening and accessibility review.
