// ============================================
// Service Worker for RustChat
// Provides offline support and caching
// ============================================

const CACHE_NAME = 'rustchat-v2';
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/favicon.ico',
];

// ============================================
// Install Event
// ============================================

self.addEventListener('install', (event) => {
  console.log('[SW] Installing...');

  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      console.log('[SW] Caching static assets');
      return cache.addAll(STATIC_ASSETS);
    }).then(() => {
      return self.skipWaiting();
    })
  );
});

// ============================================
// Activate Event
// ============================================

self.addEventListener('activate', (event) => {
  console.log('[SW] Activating...');

  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => {
            console.log('[SW] Deleting old cache:', name);
            return caches.delete(name);
          })
      );
    }).then(() => {
      return self.clients.claim();
    })
  );
});

// ============================================
// Fetch Event - Cache Strategies
// ============================================

self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Only handle same-origin requests.
  if (url.origin !== self.location.origin) {
    return;
  }

  // Skip API requests (don't cache dynamic data)
  if (url.pathname.startsWith('/api/')) {
    return;
  }

  // Skip WebSocket requests
  if (url.protocol === 'wss:' || url.protocol === 'ws:') {
    return;
  }

  // Hashed build assets must prefer network to avoid stale chunk errors
  // after deployments (dynamic import 404/mismatch).
  if (isBuildAsset(url)) {
    event.respondWith(networkFirst(request));
    return;
  }

  // Strategy: Network First for pages
  if (isPage(request)) {
    event.respondWith(networkFirst(request));
    return;
  }

  // Other static files can stay cache-friendly.
  if (isStaticAsset(url)) {
    event.respondWith(staleWhileRevalidate(request));
    return;
  }

  // Default fallback.
  event.respondWith(networkFirst(request));
});

// ============================================
// Cache Strategies
// ============================================

/**
 * Cache First Strategy
 * Use cached version, fall back to network
 */
async function cacheFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);

  if (cached) {
    return cached;
  }

  try {
    const response = await fetch(request);
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.error('[SW] Fetch failed:', error);
    return new Response('Network error', { status: 408 });
  }
}

/**
 * Network First Strategy
 * Try network first, fall back to cache
 */
async function networkFirst(request) {
  const cache = await caches.open(CACHE_NAME);

  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.log('[SW] Network failed, serving from cache');
    const cached = await cache.match(request);
    if (cached) {
      return cached;
    }
    throw error;
  }
}

/**
 * Stale While Revalidate Strategy
 * Serve from cache immediately, update cache in background
 */
async function staleWhileRevalidate(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);

  const fetchPromise = fetch(request).then((networkResponse) => {
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  }).catch((error) => {
    console.error('[SW] Background fetch failed:', error);
    throw error;
  });

  return cached || fetchPromise;
}

// ============================================
// Helpers
// ============================================

function isBuildAsset(url) {
  return url.pathname.startsWith('/assets/');
}

function isStaticAsset(url) {
  const staticExtensions = [
    '.png', '.jpg', '.jpeg', '.gif', '.svg', '.ico', '.woff', '.woff2', '.ttf', '.eot',
    '.json', '.webmanifest'
  ];
  return staticExtensions.some((ext) => url.pathname.endsWith(ext));
}

function isPage(request) {
  return request.mode === 'navigate' || 
         request.headers.get('accept')?.includes('text/html');
}

// ============================================
// Background Sync (for message sending)
// ============================================

self.addEventListener('sync', (event) => {
  if (event.tag === 'send-messages') {
    event.waitUntil(sendPendingMessages());
  }
});

async function sendPendingMessages() {
  // TODO: Implement background sync for pending messages
  console.log('[SW] Background sync: sending pending messages');
}

// ============================================
// Push Notifications (placeholder)
// ============================================

self.addEventListener('push', (event) => {
  if (!event.data) return;

  const data = event.data.json();
  
  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: '/icon-192.png',
      badge: '/badge-72.png',
      tag: data.tag,
      data: data.data,
    })
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  event.waitUntil(
    self.clients.matchAll({ type: 'window' }).then((clientList) => {
      if (clientList.length > 0) {
        clientList[0].focus();
      } else {
        self.clients.openWindow('/');
      }
    })
  );
});

console.log('[SW] Service Worker loaded');
