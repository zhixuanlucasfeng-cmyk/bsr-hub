import assert from "node:assert/strict";
import test from "node:test";
import { footerGroups } from "./footer-links.ts";

test("every footer destination is a valid internal or HTTPS link", () => {
  for (const group of footerGroups) {
    assert.ok(group.title.length > 0);
    for (const link of group.links) {
      assert.ok(link.label.length > 0);
      if (link.external) assert.match(link.href, /^https:\/\//);
      else assert.match(link.href, /^(\/|#)/);
    }
  }
});

test("marketplace links expose all three business filters", () => {
  const marketplace = footerGroups.find((group) => group.title === "Marketplace")!;
  assert.deepEqual(marketplace.links.map((link) => link.href), [
    "/#market",
    "/?type=rental#market",
    "/?type=workspace#market",
    "/?type=sale#market",
  ]);
});

test("trust and impact destinations include help, official Babson, contact, and project URLs", () => {
  const hrefs = footerGroups.flatMap((group) => group.links.map((link) => link.href));
  assert.ok(hrefs.includes("/help/#protected-payment"));
  assert.ok(hrefs.includes("/help/#terms"));
  assert.ok(hrefs.includes("/help/#privacy"));
  assert.ok(hrefs.includes("https://www.babson.edu/summer-at-babson/high-school-learners/summer-study/"));
  assert.ok(hrefs.includes("https://github.com/zhixuanlucasfeng-cmyk/bsr-hub/issues"));
  assert.ok(hrefs.includes("https://github.com/zhixuanlucasfeng-cmyk/bsr-hub"));
});
