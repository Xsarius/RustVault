/**
 * 404 page — shown for unmatched routes.
 */

import { A } from "@solidjs/router";
import { Button } from "~/components/ui";

export default function NotFoundPage() {
  return (
    <div class="min-h-screen flex items-center justify-center bg-bg px-4">
      <div class="text-center">
        <p class="text-6xl font-bold text-text-tertiary mb-2">404</p>
        <h1 class="text-xl font-semibold text-text mb-2">Page not found</h1>
        <p class="text-sm text-text-secondary mb-6">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <A href="/">
          <Button variant="primary" size="sm">
            Go Home
          </Button>
        </A>
      </div>
    </div>
  );
}
