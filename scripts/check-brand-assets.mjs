import { access, readFile, stat } from "node:fs/promises";
import { constants } from "node:fs";

const requirements = [
  ["apps/web/src/components/BrandLogo.tsx", ["NEXT_PUBLIC_BASE_PATH"]],
  ["apps/runner/src/components/BrandLogo.tsx", ["NEXT_PUBLIC_BASE_PATH"]],
  ["apps/web/src/app/layout.tsx", ["bsr-icon.png"]],
  ["apps/runner/src/app/layout.tsx", ["bsr-icon.png"]],
];

const assets = [
  "apps/web/public/brand/bsr-icon.svg",
  "apps/web/public/brand/bsr-icon.png",
  "apps/web/public/brand/bsr-hub-logo.svg",
  "apps/runner/public/brand/bsr-icon.svg",
  "apps/runner/public/brand/bsr-icon.png",
  "apps/runner/public/brand/bsr-runner-logo.svg",
];

const failures = [];

for (const [file, markers] of requirements) {
  try {
    const source = await readFile(file, "utf8");
    for (const marker of markers) {
      if (!source.includes(marker)) failures.push(`${file} is missing ${marker}`);
    }
  } catch {
    failures.push(`${file} does not exist`);
  }
}

for (const asset of assets) {
  try {
    await access(asset, constants.R_OK);
    if ((await stat(asset)).size === 0) failures.push(`${asset} is empty`);
  } catch {
    failures.push(`${asset} does not exist or is unreadable`);
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Brand asset contract passed.");
