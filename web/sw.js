// Capitol Dungeon service worker: cache-first so the game runs offline
// once installed. Bump CACHE on each release to invalidate old assets.
const CACHE = 'capitol-dungeon-v1784905742';
const ASSETS = [
  '.',
  'index.html',
  'mq_js_bundle.js',
  'capitol-dungeon.wasm',
  'manifest.webmanifest',
  'icon-192.png',
  'icon-512.png',
  'icon-180.png',
  'data/mobs.json',
  'data/items.json',
  'data/npcs.json',
  'data/spells.json',
  'data/achievements.json',
  'data/graffiti.json',
  'data/banter.json',
];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE).then((c) => c.addAll(ASSETS)).then(() => self.skipWaiting()));
});

self.addEventListener('activate', (e) => {
  e.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

// network-first: always fresh when online, cached copy keeps it playable offline
self.addEventListener('fetch', (e) => {
  e.respondWith(
    fetch(e.request).then((res) => {
      const copy = res.clone();
      caches.open(CACHE).then((c) => c.put(e.request, copy));
      return res;
    }).catch(() => caches.match(e.request, { ignoreSearch: true }))
  );
});
