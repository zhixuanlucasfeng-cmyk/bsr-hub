# BSR Hub — Presentation Defense Questions and Suggested Answers

These answers describe the approved two-week MVP and clearly label future plans. Team members should answer naturally rather than memorize every word.

## 1. Why would users choose BSR Hub instead of an existing marketplace?

BSR Hub combines product rentals, second-hand sales, and workspace bookings in one local community platform. A user can rent a PS5, buy a used computer, or reserve a printing facility with one account. Each listing can support pickup, owner-arranged delivery, owner-location use, or on-site use. The focus is affordable access rather than requiring everyone to buy new products.

## 2. What happens if a rented PS5 is damaged or not returned?

The owner sets a deposit, and both sides record the item's condition at handoff and return. The platform keeps the order, payment, and status history. A normal damage disagreement follows the platform dispute process; serious suspected theft or fraud may require the user to contact law enforcement. Future versions can add identity verification, insurance, and professional dispute handling. The MVP only simulates deposits in Stripe test mode.

## 3. How can five people complete so many marketplace functions in two weeks?

Rental, sale, and workspace booking reuse one account, listing, search, order, payment, fulfillment, and review foundation. Delivery is an owner-arranged order option, not a separate courier marketplace. The team has separate ownership, must complete the PS5 rental by Day 6, and stops adding features after Day 8.

## 4. How will BSR Hub make money without hurting low-income users?

The MVP demonstrates a configurable service fee. A reasonable starting example is 6% of a successful transaction, displayed before payment. Publishing and browsing remain free. Future revenue may include promoted listings, business memberships, or local advertising. The team must test whether the total remains significantly cheaper than buying new.

## 5. How will the platform obtain its first listings and users?

Start with Babson and the nearby community. Invite team members, local second-hand businesses, and relevant content creators to publish early listings with temporary fee incentives. Recruit early customers through student clubs, social media, referrals, and a waiting list. Do not claim a partnership until the partner has actually agreed.

## 6. How will BSR Hub protect private addresses?

Public pages show only city or approximate area. Exact pickup or on-site addresses are visible only to authorized buyer and seller accounts after payment and seller confirmation. Database permissions restrict address access, and private phone numbers do not appear publicly. The user agreement supports these rules but does not replace technical access controls.

## 7. How is a fair rental price determined?

The owner chooses the final price. BSR Hub can recommend a range based on replacement price, age, condition, accessories, rental duration, nearby comparable listings, demand, delivery, and risk. AI may improve recommendations after enough real transaction data exists, but the new platform should not pretend an unsupported AI price is automatically fair.

## 8. Why use both Next.js and Rust?

Next.js allows rapid development of the responsive website. Rust owns rules where inconsistency would cause financial or trust problems: authoritative price calculation, deposits and fees, availability checks, transactional booking creation, valid order transitions, and Stripe webhook verification. Technical ambition is a benefit, but Rust must solve these concrete problems.

## 9. What if the network or a service fails during the presentation?

The team checks Vercel, Render, Supabase, and Stripe before presenting and uses prepared test accounts and data. It also keeps a full backup recording and screenshots. If a service fails, the UI shows a real error rather than fake success, and the team can demonstrate the recorded journey, source code, database, and architecture.

## 10. How will the team measure UN Goals 8 and 10?

For Goal 8, track owner earnings, small-business bookings, and hours of previously idle resources used. For Goal 10, estimate user savings compared with buying new and count affordable rentals, sales, and space bookings. Use optional feedback rather than requiring users to disclose income. These are future measurement targets, not outcomes the MVP has already achieved.

## 11. What is the largest project risk?

Fraud and loss of trust are the largest risks because the marketplace involves strangers, valuable items, and sometimes private locations. Priorities include transaction records, deposits, condition photos, ratings, address privacy, reporting, blocking, and account enforcement. Identity verification and insurance are later-stage protections.

## 12. How will the team prove people want this product?

Interview at least 20 potential users, collect at least 10 clear rental or booking intentions, and identify at least five potential owners. Use a simple landing page or waitlist with example listings and no real charge. Run a small manually supervised pilot using team-owned items before scaling.

## 13. What if two users reserve the same PS5 at the same time?

The webpage is not authoritative. Rust checks PostgreSQL inside a transaction and creates a 30-minute pending-payment reservation. Only one overlapping request can succeed. The second user receives an unavailable response and no payment link. Successful payment confirms the reservation; non-payment expires it.

## 14. How will BSR Hub stop users from holding many items without paying?

Limit the number of simultaneous pending-payment reservations per account, rate-limit repeated order creation, expire holds after 30 minutes, and restrict accounts with repeated abusive behavior. Sellers can report or cancel suspicious requests, but manual seller action is not the primary technical protection.

## 15. What happens if a seller cancels after the buyer pays?

The buyer receives a full refund through the payment provider. The platform records the cancellation reason and affects the seller's reliability score. Repeated cancellations can reduce visibility or suspend the account. The MVP simulates payment, refund, and payout states; a real launch would need compliant marketplace payment infrastructure.

## 16. Who is responsible if someone is injured in a workshop or while using a tool?

BSR Hub must not promise that a user agreement removes all liability. Listings should disclose hazards, required training, protective equipment, age or skill restrictions, and site rules. Owners must confirm that spaces and equipment are lawful and reasonably safe. Users acknowledge instructions and report incidents. High-risk categories may require verification, insurance, or prohibition. Before a real launch, the company needs U.S. legal and insurance advice; the class MVP does not enable real high-risk transactions.

## 17. What anti-fraud practices are inspired by Dubizzle?

Use verified email or phone, listing review, report and block actions, internal communication, hidden phone numbers, prohibited-item rules, phishing education, and account suspension. Dubizzle's official guidance supports these trust patterns. BSR Hub should adapt them to U.S. users and its rental model.

## 18. Does Dubizzle hold buyer money until a transaction finishes?

Dubizzle's official safety documentation says it generally does not participate in buyer–seller payments and warns about fake messages claiming it collected payment. BSR Hub's proposed delayed seller payout is therefore its own future model, not a copied Dubizzle feature. A real implementation should use Stripe Connect or another compliant provider; the MVP only simulates it.

## 19. How will BSR Hub handle fake delivery or payment links?

Keep transaction communication inside the platform, warn users not to enter payment data on external courier links, show the official payment status only inside the order page, and provide report/block controls. The platform never asks a seller to enter card details merely to receive money.

## 20. What products are prohibited?

The platform needs an explicit list covering illegal goods, weapons, drugs, stolen or counterfeit items, personal information, financial products, hazardous materials, and other restricted categories. Reports trigger review, listing removal, and proportionate account enforcement. The MVP should seed only ordinary electronics, tools, creative equipment, furniture, and legitimate workspaces.

## 21. How will BSR Hub respond to reports?

Users select a reason, provide details, and may block the other account. The platform records the report, hides or pauses high-risk content when appropriate, reviews evidence, and chooses warning, removal, suspension, or permanent ban. Decisions and appeals should be documented for consistency.

## 22. How will buyers judge whether a seller is reliable?

Show completed-order ratings, response behavior, cancellation rate, account age, and future verification badges. Do not imply that a badge guarantees safety. Encourage complete photos and descriptions, and provide listing insights that help legitimate owners improve quality.

## 23. Why not let users complete payment through WhatsApp or private links?

External communication makes phishing, fake courier links, and false payment screenshots harder to detect. Keeping the order and payment status inside BSR Hub produces an audit trail and makes reporting easier. Users should never share passwords, OTPs, card numbers, or banking credentials with another user.

## 24. What if the platform has too few listings in one category?

Launch in one local area, measure searches that return no results, and recruit supply for the categories with demonstrated demand. Notify waitlisted users when a suitable listing appears. Do not expand to more cities until the local marketplace has enough supply and completed transactions.

## 25. When is BSR Hub ready for a real launch?

Not after the class demo alone. A real launch requires live payment and payout compliance, identity and fraud controls, refund/dispute policies, prohibited-item enforcement, insurance and liability review, customer support, privacy/legal documents, security testing, and a controlled local pilot.

## Research Sources

The Dubizzle-derived answers are based on its official Help Center. See [Dubizzle research and lessons](Dubizzle-Research-Notes.md) for direct source links and the important difference between its marketplace model and BSR Hub's proposed payment model.
