# Order Transition Matrix

| Current | Action | Next | Actor in UI |
|---|---|---|---|
| pending_payment | mark_paid | paid | buyer/mock payment |
| pending_payment | expire | expired | server |
| pending_payment, paid, confirmed | cancel | cancelled | participant |
| paid | confirm | confirmed | seller |
| confirmed | activate | active | seller |
| confirmed | fulfill | fulfilled | seller (sale/workspace) |
| active | return | returned | buyer |
| returned, fulfilled | complete | completed | seller |

Every other pair is rejected with HTTP 409 in demo mode or a stable conflict error in production mode.
