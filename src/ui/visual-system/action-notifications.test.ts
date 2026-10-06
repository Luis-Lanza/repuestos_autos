import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { ActionNotificationProvider, useActionNotifications } from "./action-notifications.ts";

test("provider renders children without a browser portal during server rendering", () => {
  function Consumer() {
    const notifications = useActionNotifications();
    return createElement("p", null, notifications ? "Shared host available" : "Inline fallback");
  }
  assert.match(renderToStaticMarkup(createElement(ActionNotificationProvider, null, createElement(Consumer))), /Shared host available/);
  assert.match(renderToStaticMarkup(createElement(Consumer)), /Inline fallback/);
});

test("host styles fix top-right position and semantic severity colors independently of shell clipping", async () => {
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-action-notifications\] \{[^}]*position: fixed;[^}]*inset-block-start:[^}]*inset-inline-end:/);
  for (const severity of ["success", "error", "warning", "info"]) {
    assert.match(css, new RegExp(`\\[data-ui-action-notice\\]\\[data-severity="${severity}"\\] \\{[^}]*background:`));
  }
  assert.match(css, /\[data-ui-notification-live\] \{[^}]*clip-path: inset\(50%\)/);
});
