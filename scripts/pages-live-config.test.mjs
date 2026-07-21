import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const buildScript = await readFile(new URL("./build-pages.sh", import.meta.url), "utf8");
const workflow = await readFile(
  new URL("../.github/workflows/pages.yml", import.meta.url),
  "utf8",
);
const marketplacePage = await readFile(
  new URL("../apps/web/src/app/page.tsx", import.meta.url),
  "utf8",
);
const bookingCard = await readFile(
  new URL("../apps/web/src/components/BookingCard.tsx", import.meta.url),
  "utf8",
);
const authConfig = await readFile(
  new URL("../apps/web/src/lib/auth-config.ts", import.meta.url),
  "utf8",
);

assert.match(
  buildScript,
  /hub_static_demo="\$\{NEXT_PUBLIC_STATIC_DEMO:-true\}"/,
  "Hub must fall back to demo mode when the deployment secret is absent",
);
assert.match(
  buildScript,
  /NEXT_PUBLIC_STATIC_DEMO="\$hub_static_demo"[\s\S]*NEXT_PUBLIC_BASE_PATH=\/bsr-hub\/hub/,
  "Hub build must consume the configured live/demo flag",
);
assert.match(
  buildScript,
  /NEXT_PUBLIC_STATIC_DEMO=true[\s\S]*NEXT_PUBLIC_BASE_PATH=\/bsr-hub\/runner/,
  "Runner must remain a static demo",
);

for (const secret of [
  "NEXT_PUBLIC_SUPABASE_URL",
  "NEXT_PUBLIC_SUPABASE_ANON_KEY",
  "NEXT_PUBLIC_API_BASE_URL",
  "NEXT_PUBLIC_STATIC_DEMO",
]) {
  assert.match(
    workflow,
    new RegExp(`${secret}: \\$\\{\\{ secrets\\.${secret} \\}\\}`),
    `GitHub Pages workflow must expose ${secret}`,
  );
}

for (const source of [marketplacePage, bookingCard]) {
  assert.match(
    source,
    /NEXT_PUBLIC_API_BASE_URL\s*\?\?\s*process\.env\.NEXT_PUBLIC_API_URL/,
    "Every live marketplace API call must prefer NEXT_PUBLIC_API_BASE_URL",
  );
}

for (const variable of [
  "NEXT_PUBLIC_SUPABASE_URL",
  "NEXT_PUBLIC_SUPABASE_ANON_KEY",
  "NEXT_PUBLIC_API_BASE_URL",
  "NEXT_PUBLIC_STATIC_DEMO",
]) {
  assert.match(
    authConfig,
    new RegExp(`publicEnv[\\s\\S]*process\\.env\\.${variable}`),
    `${variable} must be read directly so Next.js can inline it in browser code`,
  );
}

console.log("GitHub Pages live-account configuration is wired correctly.");
