import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const buildScript = await readFile(new URL("./build-pages.sh", import.meta.url), "utf8");
const workflow = await readFile(
  new URL("../.github/workflows/pages.yml", import.meta.url),
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

console.log("GitHub Pages live-account configuration is wired correctly.");
