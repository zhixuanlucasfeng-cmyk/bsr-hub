# Lucian — Individual Task Instructions

You do not need Claude or any coding agent for this role. Your job is research, realistic marketplace content, and organizing the team's files so Lucas can integrate them safely.

## 1. Research Task

Research examples of services for:

- renting products such as consoles, computers, cameras, and tools;
- booking studios, workshops, printing facilities, or small factories;
- buying and selling second-hand products;
- pickup and local delivery.

For each useful example, record:

- company or service name;
- website link;
- what users can rent, book, buy, or sell;
- how pricing works;
- how pickup or delivery works;
- one feature BSR Hub should learn from;
- one problem BSR Hub should avoid.

Write the results in a Google Doc, Word file, spreadsheet, or Markdown file. Do not copy long text from other websites; summarize it in your own words.

## 2. Demo Listing Content

Prepare at least twelve realistic U.S. listings:

- two PS5 or gaming listings;
- two computer or electronics listings;
- two camera or creative-equipment listings;
- two tools or maker-equipment listings;
- two studios, printing facilities, workshops, or small factories;
- two second-hand products for sale.

Each listing must include:

- title;
- type: rental, sale, or workspace;
- category;
- short description;
- price and pricing unit;
- deposit for rentals, if appropriate;
- condition;
- city and state near Babson;
- pickup, delivery, owner-location use, or on-site use;
- suggested delivery fee, if offered;
- image-search suggestion or original photo filename.

Do not include a real private home address, phone number, payment detail, or other sensitive information.

## 3. Daily File Collection

At the end of every workday, ask Lucas, Anna, Yichen, and Nasia for their newest project files or exported folders. Create this structure:

```text
BSR-Hub-Team-Handoff/
  YYYY-MM-DD/
    Lucas/
    Anna/
    Yichen/
    Nasia/
    Lucian-Research/
    INVENTORY.md
```

Place each person's files only inside that person's folder. Never overwrite yesterday's folder. If someone sends a replacement on the same day, add a suffix such as `v2` and update the inventory.

## 4. Files You Must Not Collect

Do not collect or share:

- `.env` files or API keys;
- passwords or Stripe/Supabase secrets;
- `node_modules/`;
- `.next/`, `dist/`, `build/`, or Rust `target/` folders;
- personal addresses or identity documents;
- unrelated personal files.

These files are either unsafe to share or can be regenerated.

## 5. Inventory Format

Create `INVENTORY.md` inside each day's folder:

```markdown
# BSR Hub Daily Handoff — YYYY-MM-DD

| Person | Received | Main files/features | Version | Missing or blocked |
|---|---|---|---|---|
| Lucas | Yes/No | Rust API, integration | v1 | ... |
| Anna | Yes/No | Frontend and forms | v1 | ... |
| Yichen | Yes/No | Database and tests | v1 | ... |
| Nasia | Yes/No | Checkout and demo | v1 | ... |
| Lucian | Yes/No | Research and listings | v1 | ... |
```

## 6. Handoff to Lucas

1. Confirm every received folder appears in `INVENTORY.md`.
2. Confirm no secret or generated folders are included.
3. Compress the dated folder as `BSR-Hub-Handoff-YYYY-MM-DD.zip`.
4. Send the ZIP file and the inventory to Lucas through the team's agreed channel.
5. Keep the previous day's package as a backup.

Lucas, not Lucian, decides what enters the official Git repository. Do not rename source files, combine code manually, or resolve conflicts yourself.

## 7. Final Deliverables

By Day 8, provide Lucas with:

- competitor and user research summary;
- twelve or more realistic listing records;
- pricing and privacy notes;
- one organized handoff package for each workday;
- a final inventory confirming that all team outputs were delivered.
