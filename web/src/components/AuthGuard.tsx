/**
 * Auth route guard — redirects unauthenticated users to /login.
 * On mobile, also ensures a server URL has been configured first.
 */

import { type ParentComponent, Show, onMount, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authStore, serverStore } from "~/stores";
import { isMobile } from "~/mobile/useMobile";
import { DashboardSkeleton } from "~/components/ui";

export const AuthGuard: ParentComponent = (props) => {
  const navigate = useNavigate();
  const [ready, setReady] = createSignal(false);

  onMount(async () => {
    // Always initialise the server store so the base URL is applied to the
    // API client before any fetch occurs.
    await serverStore.init();

    // On native mobile, require a server URL before allowing login.
    if (isMobile() && !serverStore.isConfigured()) {
      navigate("/server-setup", { replace: true });
      return;
    }

    await authStore.restoreSession();
    if (!authStore.isAuthenticated()) {
      navigate("/login", { replace: true });
    }
    setReady(true);
  });

  return (
    <Show when={ready() && authStore.isAuthenticated()} fallback={<DashboardSkeleton />}>
      {props.children}
    </Show>
  );
};
