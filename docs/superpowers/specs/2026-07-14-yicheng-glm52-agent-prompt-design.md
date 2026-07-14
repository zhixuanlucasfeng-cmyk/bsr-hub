# Yicheng GLM-5.2 Agent Prompt Design

**Date:** 2026-07-14  
**Owner:** Lucas  
**Operator:** Yicheng

## Objective

Create a bilingual, copy-ready instruction package that makes Yicheng's coding agent first rebuild his database/security handoff against the real BSR Hub repository, then independently audit the complete project. The workflow must maximize evidence and compatibility rather than maximize the amount of generated code.

## Model-Identity Verification

Yicheng must verify the upstream model before starting work. Asking the model what it is does not count because a model can repeat its prompt or display alias. Verification requires all three of the following, with API keys redacted:

1. The coding tool's active model selector or status output shows GLM-5.2.
2. The provider configuration maps the tool's active model to the exact GLM-5.2 model identifier and uses the official Coding Plan endpoint.
3. A fresh session's request log, provider usage record, or API response reports the same upstream model identifier.

If the official account model list does not offer GLM-5.2, or the response reports GLM-5.1/GLM-4.7, the agent must stop and report that it cannot prove GLM-5.2 is active. Credentials must never be pasted into chat, screenshots, logs, or the handoff archive.

## Workflow

### Phase 1: Repair Yicheng's handoff

The agent starts from current `main` at or after commit `5bf9bb5`. It reads `AGENTS.md`, all ordered migrations, the Rust domain and repository ports, the API contracts, the existing Yicheng design, and relevant tests before proposing changes.

It must not replace migration 001, invent fields, duplicate canonical payment data, weaken Lucas's pricing/payment rules, or claim tests that were not executed. It rewrites migrations 002 and 003 and their SQL tests to extend the actual schema. The phase ends only with migration evidence, RLS evidence, booking-conflict evidence, a clean diff, and a dedicated Git commit.

### Phase 2: Independent project audit

After Phase 1 passes, the agent re-reads the updated repository and audits Rust, SQL, Supabase RLS, Stripe processing, API contracts, data privacy, concurrency, and test coverage. Each finding must cite an existing file and exact line, include reproduction evidence, distinguish confirmed defects from suggestions, and account for protections already implemented elsewhere.

The agent may make only bounded fixes supported by a failing test. Broad refactors, new product features, dependency upgrades, schema replacement, deployment, and live external operations are out of scope. The phase ends with a second Git commit, full quality-gate output, and a remaining-risk report.

## Safety and Quality Gates

- Preserve unrelated user changes and work on a named feature branch.
- Never read or package `.env` files, tokens, API keys, private addresses, payment credentials, `.git`, caches, or build output.
- Use integer cents and server-authoritative prices and billable units.
- Keep the seller adjustment within ±500 cents per billing unit.
- Preserve Stripe signature, type, paid-status, order, amount, currency, expiry, and idempotency checks.
- Do not expose exact addresses through public RLS policies.
- Harden every `SECURITY DEFINER` function with a fixed safe `search_path` and minimal grants.
- Prove RLS tests under anonymous/authenticated roles rather than only a privileged database role.
- Run formatting, Clippy with denied warnings, the complete Rust test suite, SQL tests when a disposable local database exists, migration parsing, and `git diff --check`.
- Report a test as `NOT RUN` when its required environment is unavailable.

## Deliverable

The final prompt package will contain:

1. A short operator checklist for confirming GLM-5.2.
2. A Chinese operator explanation.
3. An English master prompt optimized for direct use by the coding agent.
4. Phase gates and required evidence formats.
5. A final response template and safe handoff archive checklist.

## Success Criteria

The package succeeds when Yicheng can prove the active model, the agent changes only repository-compatible files, both phases are independently reviewable, every claimed verification has captured command output, and Lucas receives a secret-free handoff that can be integrated without replacing the established schema or transaction rules.
