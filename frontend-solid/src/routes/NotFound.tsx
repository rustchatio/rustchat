import { useNavigate } from '@solidjs/router';
import Button from '../components/ui/Button';

export default function NotFound() {
  const navigate = useNavigate();

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app px-4">
      <div class="text-center max-w-md">
        {/* 404 Icon */}
        <div class="mx-auto h-24 w-24 bg-bg-panel rounded-2xl flex items-center justify-center mb-6 border border-border-1">
          <span class="text-5xl">🤔</span>
        </div>

        <h1 class="text-6xl font-bold text-text-1 mb-2">404</h1>
        <h2 class="text-2xl font-semibold text-text-2 mb-4">Page Not Found</h2>
        <p class="text-text-3 mb-8">
          The page you're looking for doesn't exist or has been moved.
        </p>

        <div class="flex flex-col sm:flex-row gap-3 justify-center">
          <Button variant="primary" onClick={() => navigate(-1)}>
            Go Back
          </Button>
          <Button variant="secondary" onClick={() => navigate('/')}>
            Go Home
          </Button>
        </div>
      </div>
    </div>
  );
}
