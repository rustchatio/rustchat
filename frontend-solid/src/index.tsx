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

render(() => <App />, root);
