/**
 * App root — sets up the router with lazy-loaded pages.
 */

import { lazy, type Component } from "solid-js";
import { Router, Route } from "@solidjs/router";
import { AppShell } from "~/components/layout";
import { AuthGuard } from "~/components/AuthGuard";
import { ToastRegion } from "~/components/ui";

// ── Lazy-loaded pages ────────────────────────────────────────

const Login = lazy(() => import("~/pages/Login"));
const Register = lazy(() => import("~/pages/Register"));
const OidcCallback = lazy(() => import("~/pages/OidcCallback"));

const Dashboard = lazy(() => import("~/pages/Dashboard"));
const Transactions = lazy(() => import("~/pages/Transactions"));
const Banks = lazy(() => import("~/pages/Banks"));
const Categories = lazy(() => import("~/pages/Categories"));
const Tags = lazy(() => import("~/pages/Tags"));
const Budget = lazy(() => import("~/pages/Budget"));
const Reports = lazy(() => import("~/pages/Reports"));
const Settings = lazy(() => import("~/pages/Settings"));
const More = lazy(() => import("~/pages/More"));
const NotFound = lazy(() => import("~/pages/NotFound"));

// ── Protected layout wrapper ─────────────────────────────────

const ProtectedLayout: Component<{ children?: any }> = (props) => (
  <AuthGuard>
    <AppShell>{props.children}</AppShell>
  </AuthGuard>
);

// ── App ──────────────────────────────────────────────────────

const App: Component = () => {
  return (
    <>
      <Router>
        {/* Public auth routes — no shell */}
        <Route path="/login" component={Login} />
        <Route path="/register" component={Register} />
        <Route path="/auth/oidc/callback" component={OidcCallback} />

        {/* Protected routes — inside AppShell */}
        <Route path="/" component={ProtectedLayout}>
          <Route path="/" component={Dashboard} />
          <Route path="/transactions" component={Transactions} />
          <Route path="/banks" component={Banks} />
          <Route path="/categories" component={Categories} />
          <Route path="/tags" component={Tags} />
          <Route path="/budget" component={Budget} />
          <Route path="/reports" component={Reports} />
          <Route path="/settings" component={Settings} />
          <Route path="/more" component={More} />
        </Route>

        {/* Catch-all */}
        <Route path="*" component={NotFound} />
      </Router>

      <ToastRegion />
    </>
  );
};

export default App;
