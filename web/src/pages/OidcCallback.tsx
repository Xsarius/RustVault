/**
 * OIDC callback page — handles the redirect from the OIDC provider.
 */

import { onMount } from "solid-js";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { api } from "~/api";
import { authStore } from "~/stores";
import { Skeleton } from "~/components/ui";

export default function OidcCallbackPage() {
  const navigate = useNavigate();
  const [params] = useSearchParams();

  onMount(async () => {
    const code = params.code;
    const state = params.state;

    if (!code || !state) {
      navigate("/login", { replace: true });
      return;
    }

    try {
      const res = await api.post<{
        data: { access_token: string; refresh_token: string };
      }>("/api/auth/oidc/callback", { code, state });

      api.setTokens(res.data.access_token, res.data.refresh_token);
      await authStore.fetchMe();
      navigate("/", { replace: true });
    } catch {
      navigate("/login", { replace: true });
    }
  });

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg">
      <div class="text-center space-y-4">
        <div class="flex justify-center">
          <Skeleton variant="circle" class="w-12 h-12" />
        </div>
        <p class="text-sm text-text-secondary">Completing sign-in…</p>
      </div>
    </div>
  );
}
