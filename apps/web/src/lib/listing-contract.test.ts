import assert from "node:assert/strict";
import test from "node:test";
import * as staticDemo from "./static-demo.ts";

test("API listings missing media are completed from the local demo catalog", () => {
  assert.equal(typeof staticDemo.attachDemoListingMedia, "function");
  const remote = staticDemo.demoCatalog.map(({ imageSrc: _imageSrc, imageAlt: _imageAlt, ...listing }) => listing);
  const completed = staticDemo.attachDemoListingMedia!(remote);

  assert.equal(completed.length, staticDemo.demoCatalog.length);
  for (const listing of completed) {
    assert.match(listing.imageSrc, /^\/images\/listings\/.+\.jpg$/);
    assert.ok(listing.imageAlt.length > 0);
  }
});
