/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import "./app.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html?",
  );
}

render(() => <App />, root);
