/* @refresh reload */
import { render } from 'solid-js/web';
import App from './App';
import './styles/index.css';
import { installCryptoPolyfills } from './utils/cryptoPolyfills';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Root element not found');
}

installCryptoPolyfills();

// Recover from stale hashed chunks after deploy/cache mismatch.
if (typeof window !== 'undefined') {
  const chunkReloadKey = 'rustchat_chunk_reload_once';
  const chunkReloadWindowMs = 30000;
  const reloadForChunkError = () => {
    const now = Date.now();
    const lastReload = Number(sessionStorage.getItem(chunkReloadKey) || 0);
    if (Number.isFinite(lastReload) && now - lastReload < chunkReloadWindowMs) return;

    sessionStorage.setItem(chunkReloadKey, String(now));
    const url = new URL(window.location.href);
    url.searchParams.set('__chunk_reload', String(now));
    window.location.replace(url.toString());
  };

  // Allow future recoveries after the current page has remained stable.
  window.setTimeout(() => {
    const lastReload = Number(sessionStorage.getItem(chunkReloadKey) || 0);
    if (Number.isFinite(lastReload) && Date.now() - lastReload >= chunkReloadWindowMs) {
      sessionStorage.removeItem(chunkReloadKey);
    }
  }, chunkReloadWindowMs + 1000);

  window.addEventListener('vite:preloadError', (event) => {
    event.preventDefault();
    reloadForChunkError();
  });

  window.addEventListener('error', (event) => {
    const message = String((event as ErrorEvent).message || '');
    if (message.includes('Failed to fetch dynamically imported module')) {
      reloadForChunkError();
    }
  });

  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason;
    const message =
      typeof reason === 'string'
        ? reason
        : reason instanceof Error
          ? reason.message
          : String(reason || '');
    if (message.includes('Failed to fetch dynamically imported module')) {
      event.preventDefault();
      reloadForChunkError();
    }
  });
}

render(() => <App />, root);
