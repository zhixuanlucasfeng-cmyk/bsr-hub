import { mkdir, readdir, stat } from "node:fs/promises";
import { basename, extname, join } from "node:path";
import sharp from "sharp";

const publicRoot = "apps/web/public/images";
const optimizedRoot = join(publicRoot, "optimized");
const sourceDirs = [join(publicRoot, "listings"), join(publicRoot, "categories")];
const sources = (await Promise.all(sourceDirs.map(async (directory) =>
  (await readdir(directory)).filter((name) => /\.(jpe?g|png)$/i.test(name)).map((name) => join(directory, name))
))).flat();

for (const variant of ["card-sm", "card-lg", "detail"]) {
  await mkdir(join(optimizedRoot, variant), { recursive: true });
}

let originalBytes = 0;
let optimizedBytes = 0;

for (const input of sources) {
  const stem = basename(input, extname(input));
  const cardSmall = join(optimizedRoot, "card-sm", `${stem}.webp`);
  const cardLarge = join(optimizedRoot, "card-lg", `${stem}.webp`);
  const detail = join(optimizedRoot, "detail", `${stem}.webp`);

  originalBytes += (await stat(input)).size;
  await sharp(input).rotate().resize(480, 360, { fit: "cover", position: "attention" }).webp({ quality: 76 }).toFile(cardSmall);
  await sharp(input).rotate().resize(960, 720, { fit: "cover", position: "attention" }).webp({ quality: 80 }).toFile(cardLarge);
  await sharp(input).rotate().resize({ width: 1440, withoutEnlargement: true }).webp({ quality: 82 }).toFile(detail);

  optimizedBytes += (await stat(cardSmall)).size + (await stat(cardLarge)).size + (await stat(detail)).size;
}

console.log(`Optimized ${sources.length} sources into ${sources.length * 3} WebP files.`);
console.log(`Original JPEG total: ${(originalBytes / 1048576).toFixed(2)} MiB.`);
console.log(`All responsive variants: ${(optimizedBytes / 1048576).toFixed(2)} MiB.`);
