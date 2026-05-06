// SAATHI Service Worker — Stale-While-Revalidate for shell assets
const SHELL_CACHE = "saathi-shell-v1";
const DATA_CACHE = "saathi-data-v1";

// Core shell assets that must load even offline
const SHELL_ASSETS = [
  "/",
  "/index.html",
  "/styles.css",
  "/icon-192.png",
  "/icon-512.png",
  "/apple-touch-icon.png",
  "/manifest.json",
];

self.addEventListener("install", (event) => {
  console.log("[SW] Install");
  event.waitUntil(
    caches
      .open(SHELL_CACHE)
      .then((cache) => cache.addAll(SHELL_ASSETS))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  console.log("[SW] Activate");
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((k) => k !== SHELL_CACHE && k !== DATA_CACHE)
            .map((k) => caches.delete(k))
        )
      )
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // API calls: network-first, fallback to cached data
  if (url.pathname.startsWith("/api/")) {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches.open(DATA_CACHE).then((cache) => cache.put(request, clone));
          return response;
        })
        .catch(() => caches.match(request).then((r) => r || new Response(JSON.stringify({ error: "Offline" }), { status: 503, headers: { "Content-Type": "application/json" } })))
    );
    return;
  }

  // Shell assets: stale-while-revalidate
  event.respondWith(
    caches.match(request).then((cached) => {
      const fetchPromise = fetch(request)
        .then((networkResponse) => {
          if (networkResponse.ok) {
            const clone = networkResponse.clone();
            caches.open(SHELL_CACHE).then((cache) => cache.put(request, clone));
          }
          return networkResponse;
        })
        .catch(() => cached);
      return cached || fetchPromise;
    })
  );
});
