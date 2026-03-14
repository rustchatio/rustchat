import { A } from '@solidjs/router';

export default function About() {
  return (
    <main class="p-8 max-w-4xl mx-auto">
      <h1 class="text-3xl font-bold mb-6 text-brand">About RustChat Solid.js</h1>

      <section class="card p-6 mb-8">
        <h2 class="text-xl font-semibold mb-4">Technology Stack</h2>
        <ul class="list-disc list-inside text-text-2 space-y-2">
          <li>
            <strong>Solid.js 1.9+</strong> - Fine-grained reactivity framework
          </li>
          <li>
            <strong>Vite 7+</strong> - Next generation frontend tooling
          </li>
          <li>
            <strong>TypeScript 5.9+</strong> - Type-safe development
          </li>
          <li>
            <strong>Tailwind CSS 4+</strong> - Utility-first CSS framework
          </li>
          <li>
            <strong>Kobalte</strong> - Headless UI components for Solid.js
          </li>
        </ul>
      </section>

      <section class="card p-6 mb-8">
        <h2 class="text-xl font-semibold mb-4">Features</h2>
        <ul class="list-disc list-inside text-text-2 space-y-2">
          <li>Full TypeScript support</li>
          <li>Dark mode and multiple theme support</li>
          <li>Keyboard navigation and accessibility</li>
          <li>Responsive design</li>
          <li>Code splitting and lazy loading</li>
        </ul>
      </section>

      <A href="/" class="text-brand hover:text-brand-hover underline">
        ← Back to Home
      </A>
    </main>
  );
}
