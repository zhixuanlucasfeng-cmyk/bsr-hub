import { readFile } from "node:fs/promises";

const failures = [];
const requireMarkers = async (file, markers) => {
  const source = await readFile(file, "utf8").catch(() => "");
  if (!source) failures.push(`${file} is missing or empty`);
  for (const marker of markers) {
    if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
  }
  return source;
};

const blueprint = await requireMarkers("render.yaml", [
  "plan: free",
  "healthCheckPath: /ready",
  "initialDeployHook: /usr/local/bin/mongo_bootstrap",
  "key: MONGODB_URI",
  "key: MONGODB_DATABASE",
  "value: bsr_hub",
  "value: https://zhixuanlucasfeng-cmyk.github.io",
]);

for (const forbidden of ["DATABASE_URL", "mongodb://", "mongodb+srv://", "sk_live_"]) {
  if (blueprint.includes(forbidden)) failures.push(`render.yaml must not contain ${forbidden}`);
}

await requireMarkers("services/core-api/Dockerfile", [
  "cargo build --release -p core-api --bins",
  "/usr/local/bin/core-api",
  "/usr/local/bin/mongo_bootstrap",
  'CMD ["core-api"]',
]);

await requireMarkers(".dockerignore", [
  "target",
  "node_modules",
  ".git",
  ".env",
]);

const envExample = await requireMarkers(".env.example", [
  "MONGODB_URI=",
  "MONGODB_DATABASE=bsr_hub",
  "STRIPE_SECRET_KEY=sk_test_",
]);
if (envExample.includes("DATABASE_URL=")) failures.push(".env.example still documents DATABASE_URL");

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Production deployment contract passed: Render free plan, MongoDB, readiness, and bootstrap are configured.");
