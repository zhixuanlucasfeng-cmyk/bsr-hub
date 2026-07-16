import { readdir, readFile, stat } from "node:fs/promises";
import { basename, extname, join } from "node:path";

const publicRoot = "apps/web/public/images";
const optimizedRoot = join(publicRoot, "optimized");
const variants = ["card-sm", "card-lg", "detail"];
const limits = { "card-sm": 90 * 1024, "card-lg": 180 * 1024, detail: 260 * 1024 };
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

const css = await readFile("apps/web/src/app/globals.css", "utf8");
if (css.includes("fonts.googleapis.com")) failures.push("Runtime Google Fonts import is still present");
const featured = await readFile("apps/web/src/components/FeaturedListings.tsx", "utf8");
if (!featured.includes("eager={index === 0}")) failures.push("Only the first listing should be eager");

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Performance and links contract passed: ${sources.length} sources, ${generatedCount} variants, ${generatedBytes} optimized bytes.`);
