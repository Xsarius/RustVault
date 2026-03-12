/**
 * Auth route guard — redirects unauthenticated users to /login.
 */

import { type ParentComponent, Show, onMount, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { authStore } from "~/stores";
import { DashboardSkeleton } from "~/components/ui";

export const AuthGuard: ParentComponent = (props) => {
  const navigate = useNavigate();
  const [ready, setReady] = createSignal(false);

  onMount(async () => {
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
