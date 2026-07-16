import assert from "node:assert/strict";
import test from "node:test";
import { optimizedImagePaths } from "./image-assets.ts";

test("maps a source image to every optimized variant", () => {
  assert.deepEqual(optimizedImagePaths("/images/listings/ps5-slim.jpg"), {
    small: "/images/optimized/card-sm/ps5-slim.webp",
    large: "/images/optimized/card-lg/ps5-slim.webp",
    detail: "/images/optimized/detail/ps5-slim.webp",
  });
});

test("rejects unsupported or pathless image sources", () => {
  assert.throws(() => optimizedImagePaths("remote-image"), /Unsupported image source/);
  assert.throws(() => optimizedImagePaths("/images/listings/item.gif"), /Unsupported image source/);
});
