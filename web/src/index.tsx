/* @refresh reload */
import { render } from "solid-js/web";
import { Suspense } from "solid-js";
import { I18nProvider } from "~/i18n";
import App from "./App";
import "./app.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html?",
  );
}

render(
  () => (
    <I18nProvider>
      <Suspense>
        <App />
      </Suspense>
    </I18nProvider>
  ),
  root,
);
