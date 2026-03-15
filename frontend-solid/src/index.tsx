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
  const reloadForChunkError = () => {
    if (sessionStorage.getItem(chunkReloadKey) === '1') return;
    sessionStorage.setItem(chunkReloadKey, '1');
    window.location.reload();
  };

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
