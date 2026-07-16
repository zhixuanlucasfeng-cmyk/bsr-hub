import { readdir, readFile, stat } from "node:fs/promises";
import { basename, extname, join } from "node:path";

const publicRoot = "apps/web/public/images";
const optimizedRoot = join(publicRoot, "optimized");
const variants = ["card-sm", "card-lg", "detail"];
const limits = { "card-sm": 90 * 1024, "card-lg": 180 * 1024, detail: 270 * 1024 };
const sourceDirs = [join(publicRoot, "listings"), join(publicRoot, "categories")];
const failures = [];
let generatedBytes = 0;
let generatedCount = 0;

const sources = (await Promise.all(sourceDirs.map(async (directory) =>
  (await readdir(directory)).filter((name) => /\.(jpe?g|png)$/i.test(name)).map((name) => join(directory, name))
))).flat();

for (const source of sources) {
  const stem = basename(source, extname(source));
  for (const variant of variants) {
    const output = join(optimizedRoot, variant, `${stem}.webp`);
    const info = await stat(output).catch(() => null);
    if (!info) failures.push(`Missing ${output}`);
    else {
      generatedBytes += info.size;
      generatedCount += 1;
      if (info.size > limits[variant]) failures.push(`${output} exceeds ${limits[variant]} bytes`);
    }
  }
}

const serviceWorker = await readFile("deploy/pages/sw.js", "utf8").catch(() => "");
for (const marker of [
  'const CACHE_PREFIX = "bsr-static-"',
  'request.method !== "GET"',
  'request.headers.has("authorization")',
  'url.origin !== self.location.origin',
  'request.mode === "navigate"',
  'url.pathname.includes("/_next/static/")',
  'event.waitUntil',
]) {
  if (!serviceWorker.includes(marker)) failures.push(`Service worker is missing ${marker}`);
}

for (const file of [
  "apps/web/src/components/PerformanceBoot.tsx",
  "apps/runner/src/components/PerformanceBoot.tsx",
]) {
  const source = await readFile(file, "utf8").catch(() => "");
  for (const marker of ["serviceWorker", "register", "rootPath", "NODE_ENV"]) {
    if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
  }
}

for (const [file, marker] of [
  ["apps/web/src/app/layout.tsx", "<PerformanceBoot/>"],
  ["apps/runner/src/app/layout.tsx", "<PerformanceBoot/>"],
  ["deploy/pages/index.html", "navigator.serviceWorker.register"],
  ["scripts/build-pages.sh", "dist-pages/sw.js"],
  ["apps/web/src/app/layout.tsx", 'rel="preload"'],
  ["apps/web/src/app/layout.tsx", "ps5-slim.webp"],
]) {
  const source = await readFile(file, "utf8").catch(() => "");
  if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
}

for (const file of [
  "deploy/pages/styles.css",
  "apps/web/src/app/globals.css",
  "apps/runner/src/app/globals.css",
]) {
  const source = await readFile(file, "utf8");
  if (source.includes("fonts.googleapis.com")) failures.push(`${file} still loads Google Fonts`);
}
const featured = await readFile("apps/web/src/components/FeaturedListings.tsx", "utf8");
if (!featured.includes("eager={index === 0}")) failures.push("Only the first listing should be eager");
const footer = await readFile("apps/web/src/components/SiteFooter.tsx", "utf8");
for (const marker of ["footerGroups", "href={link.href}", "FooterDestination", "footer-link-nav"]) {
  if (!footer.includes(marker)) failures.push(`Footer is missing ${marker}`);
}
const help = await readFile("apps/web/src/app/help/page.tsx", "utf8").catch(() => "");
for (const marker of ["protected-payment", "help-center", "terms", "privacy"]) {
  if (!help.includes(marker)) failures.push(`Help page is missing ${marker}`);
}
for (const [file, marker] of [
  ["apps/web/src/app/orders/page.tsx", "intent=\"orders\""],
  ["apps/web/src/app/create/page.tsx", "intent=\"create\""],
  ["apps/web/src/components/IntentRedirect.tsx", "window.location.replace"],
]) {
  const source = await readFile(file, "utf8").catch(() => "");
  if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Performance and links contract passed: ${sources.length} sources, ${generatedCount} variants, ${generatedBytes} optimized bytes.`);
