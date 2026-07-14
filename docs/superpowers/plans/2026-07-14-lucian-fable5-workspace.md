# Lucian Fable5 Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a safe VS Code workspace containing the complete tested BSR Hub code and a copy-ready Fable5 instruction for Lucian's research and handoff-tooling role.

**Architecture:** Use a Git linked worktree on its own `codex/lucian-fable5` branch so Lucian receives every tracked file from current `main` while his changes remain isolated. Add one start-here document inside that branch; Fable5 will create all role deliverables under `lucian-handoff/` only.

**Tech Stack:** Git worktrees, VS Code, Markdown, dependency-free Node.js `.mjs` tools and `node:test`.

## Global Constraints

- Do not copy `.env`, credentials, API keys, build output, dependency folders, browser data, or unrelated personal files.
- Do not modify Rust, SQL migrations, API contracts, Stripe/Supabase configuration, pricing, orders, deployment, or frontend pages.
- Work from current tested `main` at or after commit `bc92212`.
- Lucian's future work stays on branch `codex/lucian-fable5` under `lucian-handoff/`.

---

### Task 1: Create the isolated project copy

**Files:**
- Verify: `.gitignore`
- Create through Git: `.worktrees/lucian-fable5/`

**Interfaces:**
- Consumes: current `main` Git tree.
- Produces: isolated branch `codex/lucian-fable5` and a VS Code-ready project directory.

- [ ] **Step 1: Verify the main workspace is clean and `.worktrees` is ignored**

Run:

```bash
git status --short
git check-ignore -q .worktrees
```

Expected: no status output and ignore check exit code `0`.

- [ ] **Step 2: Create the linked worktree**

Run:

```bash
git worktree add .worktrees/lucian-fable5 -b codex/lucian-fable5
```

Expected: a new worktree checked out from current `main`.

- [ ] **Step 3: Prove isolation and baseline identity**

Run inside the worktree:

```bash
git branch --show-current
git rev-parse --short HEAD
git status --short
```

Expected: branch `codex/lucian-fable5`, the current main commit, and no status output.

### Task 2: Add the Fable5 start document

**Files:**
- Create: `LUCIAN-FABLE5-START-HERE.md`

**Interfaces:**
- Consumes: `docs/superpowers/specs/2026-07-14-lucian-research-tooling-design.md`.
- Produces: a self-contained operator checklist, role description, exact scope, implementation sequence, test gates, and final-report format.

- [ ] **Step 1: Write the complete instruction**

The instruction must require Fable5 to prove the repository path and commit, read `AGENTS.md` and the approved design, create only `lucian-handoff/`, use source-linked paraphrased research, implement at least twelve safe listings, write dependency-free validators and tests, avoid secrets and private data, run every available quality gate, commit real changes, and replace every report placeholder with command evidence.

- [ ] **Step 2: Review the instruction for ambiguity and unsafe scope**

Run:

```bash
rg -n "TBD|TODO|modify migrations|modify Rust|modify frontend" LUCIAN-FABLE5-START-HERE.md
git diff --check
```

Expected: no placeholders, prohibitions are explicit, and diff check passes.

- [ ] **Step 3: Commit the start document on Lucian's branch**

Run:

```bash
git add LUCIAN-FABLE5-START-HERE.md
git commit -m "docs: add Lucian Fable5 workspace instructions"
```

Expected: one new commit only on `codex/lucian-fable5`.

### Task 3: Verify and open the workspace

**Files:**
- Verify: all tracked project files in the worktree.

**Interfaces:**
- Consumes: isolated worktree with start document.
- Produces: verified VS Code window ready for Lucian/Fable5.

- [ ] **Step 1: Run the existing project quality gate**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Expected: all commands pass and all 29 Rust tests succeed.

- [ ] **Step 2: Confirm the worktree contains no untracked secrets**

Run a credential-shaped pattern scan while excluding `.git`, `target`, dependency, and build directories. Expected: no real credential-shaped values.

- [ ] **Step 3: Open the isolated folder in a new VS Code window**

Run:

```bash
code -n /Users/lucasfeng/Documents/babson/.worktrees/lucian-fable5
```

Expected: VS Code opens the Lucian workspace and displays `LUCIAN-FABLE5-START-HERE.md` in the Explorer.
