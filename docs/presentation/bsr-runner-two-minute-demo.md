# BSR Runner two-minute team demo

This script gives every team member a clear speaking part. Speak naturally; do not read the bracketed stage directions.

## 0:00–0:24 — Lucas: problem and opening

“Imagine you need to return a rented camera before class, but you do not have a car. At the same time, someone nearby has a free hour and wants flexible work. Today those two people struggle to find each other safely. BSR Runner is our solution: a local task marketplace connected to BSR Hub.”

## 0:24–0:46 — Anna: customers and communication

“Customers can post ordinary errands, package or grocery pickups, and delivery requests for BSR rentals or second-hand purchases. They describe the route, size, time and urgency. The public listing shows only general areas, while the exact addresses stay private until a runner accepts.”

## 0:46–1:08 — Yichen: logic and fair pricing

“Our Rust pricing engine calculates fair pay from distance, estimated time, weight, waiting time and urgency. Customers cannot start a bidding war that pushes workers below the automatic price. The backend also acts as a state machine: it decides which person can accept, confirm pickup, start delivery or complete each task.”

## 1:08–1:30 — Lucian: research and safety

“Our research showed that trust is the hardest part of a peer-to-peer service. BSR Runner blocks illegal, dangerous and emergency requests. It uses an age rule, address privacy, an audit trail and protected payment. This prototype uses only fictional people, addresses and money; a real U.S. launch would require legal, insurance and worker-classification review.”

## 1:30–1:53 — Nasia: product demonstration

“Here is the live flow. Maya posts a task and receives an explainable quote. Jordan sees the exact payout before accepting. After pickup and delivery, Maya enters the completion code. Only then does the simulated protected payment move to Jordan’s earnings. The admin safety desk can review applications and task activity.”

## 1:53–2:05 — all / Lucas closes

“BSR Runner supports United Nations Goal 8, decent work and economic growth, and Goal 10, reduced inequalities. We are making useful services more accessible while creating transparent local earning opportunities. That is how BSR Hub gets things moving.”

## Live-demo order

1. Show the landing page and local job cards.
2. Post a task and show the Rust price explanation.
3. Switch to runner, accept task 1, confirm pickup, and start delivery.
4. Switch to customer, enter `482731`, and complete it.
5. Show runner earnings and the admin safety desk.
