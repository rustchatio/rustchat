import { For, createSignal } from 'solid-js';
import { Button } from '@ui/Button';
import { Input } from '@ui/Input';
import { Modal } from '@ui/Modal';
import { useTheme } from '../stores/theme';

export default function Home() {
  const [isModalOpen, setIsModalOpen] = createSignal(false);
  const [inputValue, setInputValue] = createSignal('');
  const { theme, setTheme, availableThemes } = useTheme();

  return (
    <main class="p-8 max-w-4xl mx-auto">
      <h1 class="text-3xl font-bold mb-6 text-brand">RustChat Solid.js</h1>
      <p class="text-text-2 mb-8">
        Welcome to the Solid.js version of RustChat. This is the foundation phase.
      </p>

      {/* Theme Selector */}
      <section class="card p-6 mb-8">
        <h2 class="text-xl font-semibold mb-4">Theme</h2>
        <div class="flex flex-wrap gap-2">
          <For each={availableThemes}>
            {t => (
              <Button
                variant={theme() === t ? 'primary' : 'secondary'}
                onClick={() => setTheme(t)}
                aria-pressed={theme() === t}
              >
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </Button>
            )}
          </For>
        </div>
        <p class="mt-4 text-sm text-text-3">
          Current theme: <strong>{theme()}</strong>
        </p>
      </section>

      {/* Button Showcase */}
      <section class="card p-6 mb-8">
        <h2 class="text-xl font-semibold mb-4">Buttons</h2>
        <div class="flex flex-wrap gap-4">
          <Button variant="primary">Primary</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="danger">Danger</Button>
          <Button disabled>Disabled</Button>
        </div>
      </section>

      {/* Input Showcase */}
      <section class="card p-6 mb-8">
        <h2 class="text-xl font-semibold mb-4">Inputs</h2>
        <div class="space-y-4 max-w-md">
          <Input
            placeholder="Enter your name..."
            value={inputValue()}
            onInput={e => setInputValue(e.currentTarget.value)}
            label="Name"
          />
          <Input type="password" placeholder="Enter password..." label="Password" />
          <Input placeholder="Disabled input" disabled label="Disabled" />
        </div>
      </section>

      {/* Modal Showcase */}
      <section class="card p-6">
        <h2 class="text-xl font-semibold mb-4">Modal</h2>
        <Button onClick={() => setIsModalOpen(true)}>Open Modal</Button>
      </section>

      {/* Modal */}
      <Modal
        isOpen={isModalOpen()}
        onClose={() => setIsModalOpen(false)}
        title="Example Modal"
        description="This is a modal dialog with focus trapping and keyboard navigation."
      >
        <div class="space-y-4">
          <p class="text-text-2">
            This modal demonstrates proper accessibility features including:
          </p>
          <ul class="list-disc list-inside text-text-2 space-y-1">
            <li>Focus trapping</li>
            <li>Keyboard navigation (Escape to close, Tab cycling)</li>
            <li>ARIA attributes</li>
            <li>Click outside to close</li>
          </ul>
          <div class="flex gap-2 pt-4">
            <Button onClick={() => setIsModalOpen(false)}>Close</Button>
            <Button variant="primary" onClick={() => setIsModalOpen(false)}>
              Confirm
            </Button>
          </div>
        </div>
      </Modal>
    </main>
  );
}
