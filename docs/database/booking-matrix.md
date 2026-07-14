# Booking Conflict Matrix

BSR Hub uses half-open `[start, end)` ranges. Adjacent reservations are allowed; any positive overlap is rejected while either order is pending payment, paid, confirmed, or active.

| Existing order | New window | Result |
|---|---|---|
| 10:00–11:00 active | 10:00–11:00 | Reject: identical |
| 10:00–11:00 paid | 10:30–11:30 | Reject: partial overlap |
| 10:00–12:00 confirmed | 10:30–11:00 | Reject: contained |
| 10:00–11:00 pending | 11:00–12:00 | Allow: adjacent |
| 10:00–11:00 expired | 10:00–11:00 | Allow |
| 10:00–11:00 cancelled | 10:00–11:00 | Allow |
| 10:00–11:00 completed | 10:00–11:00 | Allow |

The PostgreSQL exclusion constraint is the concurrency authority. Application pre-checks improve errors but cannot replace the constraint.
