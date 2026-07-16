const CACHE_PREFIX = "bsr-static-";
const CACHE_VERSION = "2026-07-16-v1";
const ASSET_CACHE = `${CACHE_PREFIX}assets-${CACHE_VERSION}`;
const PAGE_CACHE = `${CACHE_PREFIX}pages-${CACHE_VERSION}`;

const appRoot = new URL(self.registration.scope).pathname;
const shellUrls = [appRoot, `${appRoot}hub/`, `${appRoot}runner/`];

function isSensitive(request, url) {
  return request.headers.has("authorization") || [
    "/api/", "/auth/", "/checkout/", "/payment/", "/payments/",
  ].some((segment) => url.pathname.includes(segment));
}

async function cacheFirst(request) {
  const cache = await caches.open(ASSET_CACHE);
  const cached = await cache.match(request);
  if (cached) return cached;
  const response = await fetch(request);
  if (response.ok) await cache.put(request, response.clone());
  return response;
}

async function staleWhileRevalidate(event) {
  const cache = await caches.open(ASSET_CACHE);
  const cached = await cache.match(event.request);
  const network = fetch(event.request).then(async (response) => {
    if (response.ok) await cache.put(event.request, response.clone());
    return response;
  });
  if (cached) {
    event.waitUntil(network.catch(() => undefined));
    return cached;
  }
  return network;
}

async function networkFirst(request) {
  const cache = await caches.open(PAGE_CACHE);
  try {
    const response = await fetch(request);
    if (response.ok) await cache.put(request, response.clone());
    return response;
  } catch {
    return (await cache.match(request, { ignoreSearch: true }))
      ?? (await cache.match(`${appRoot}hub/`))
      ?? Response.error();
  }
}

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(PAGE_CACHE).then((cache) => cache.addAll(shellUrls)));
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil((async () => {
    const names = await caches.keys();
    await Promise.all(names
      .filter((name) => name.startsWith(CACHE_PREFIX) && ![ASSET_CACHE, PAGE_CACHE].includes(name))
      .map((name) => caches.delete(name)));
    await self.clients.claim();
  })());
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || !url.pathname.startsWith(appRoot)) return;
  if (isSensitive(request, url)) return;

  if (request.mode === "navigate") {
    event.respondWith(networkFirst(request));
    return;
  }
  if (url.pathname.includes("/_next/static/")) {
    event.respondWith(cacheFirst(request));
    return;
  }
  if (request.destination === "image" || url.pathname.includes("/brand/")) {
    event.respondWith(staleWhileRevalidate(event));
  }
});
