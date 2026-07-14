# Lucian Research and Handoff Tooling Design

**Date:** 2026-07-14  
**Owner:** Lucian  
**Integration owner:** Lucas

## Objective

Lucian will use Fable5 to turn his research and file-coordination role into a bounded, testable contribution. The work will provide cited competitor research, safe realistic demo listings, automated listing validation, and a read-only handoff safety auditor without changing application behavior owned by other teammates.

## Ownership Boundary

Lucian may create research, demo-data, validation, audit, test, and handoff documentation files. He must not modify Rust code, SQL migrations, API contracts, Stripe or Supabase configuration, pricing rules, order logic, deployment files, or Anna and Nasia's frontend pages.

The work must start from the latest `main` after commit `df4d3c1` on a named branch. Existing unrelated changes must be preserved.

## Deliverables

### Competitor research

Create a concise, source-linked comparison covering U.S.-relevant product rental, resale, local-delivery, and workspace/studio platforms. Each entry records the service, supported transaction types, pricing pattern, fulfillment pattern, trust mechanism, one lesson for BSR Hub, one risk to avoid, source URL, and access date. Claims must be paraphrased and tied to a source; uncertain claims must be labeled.

### Demo listing data

Create at least twelve fictional U.S. listings near Babson College:

- two gaming or PS5 listings;
- two computers or electronics listings;
- two cameras or creative-equipment listings;
- two tools or maker-equipment listings;
- two studios, printing facilities, workshops, or small factories;
- two second-hand sale listings.

Each record includes a stable ID, title, listing type, category, description, integer-cent unit price, billing unit where relevant, deposit, condition, city, two-letter state, allowed fulfillment methods, delivery fee, and an image-search suggestion. Records must not contain real street addresses, phone numbers, emails, payment details, credentials, or real personal identities.

### Demo-data validator

Create a dependency-free Node.js validator and tests. It verifies JSON syntax, required fields, category counts, integer non-negative cents, valid listing types, valid billing and fulfillment values, U.S. state format, privacy restrictions, unique IDs, and category-specific rules. It exits nonzero and prints actionable record-level errors when validation fails.

### Handoff safety auditor

Create a dependency-free, read-only-by-default Node.js tool and tests. Given a handoff directory, it inventories files and rejects prohibited content or paths such as `.env`, credentials, API keys, private keys, `node_modules`, `.next`, `dist`, `build`, Rust `target`, `.git`, personal addresses, and payment credentials. It must never delete or rewrite teammate files. An explicit output option may write a generated inventory report outside the scanned content.

## Error Handling and Evidence

Tools must return exit code zero only when validation succeeds. Errors identify the affected file or listing and the violated rule without printing any detected secret value. Research failures and inaccessible sources are reported rather than invented.

Fable5 must provide the exact commands and outputs for all Node tests, demo-data validation, a safe handoff fixture, a deliberately unsafe fixture, formatting or syntax checks, and `git diff --check`. Tests use temporary fixtures and may not scan personal directories.

## Handoff Structure

```text
lucian-handoff/
├── README.md
├── competitor-research.md
├── demo-listings.json
├── demo-listings.schema.json
├── tools/
│   ├── validate-demo-listings.mjs
│   └── audit-handoff.mjs
├── tests/
│   ├── validate-demo-listings.test.mjs
│   └── audit-handoff.test.mjs
└── FINAL-REPORT.md
```

Repository-ready files may remain in this isolated directory so Lucas has one integration surface. Fable5 must not include `.env`, API keys, browser profiles, `.git`, dependency folders, build output, or unrelated personal files in the returned ZIP.

## Completion Criteria

Lucian's work is complete when the research has verifiable sources, all twelve or more listings pass the validator, invalid listing fixtures fail with clear errors, prohibited handoff fixtures are rejected without leaking their contents, safe fixtures pass, all tests and diff checks pass, and the final report contains a real commit hash and captured evidence rather than placeholders.
