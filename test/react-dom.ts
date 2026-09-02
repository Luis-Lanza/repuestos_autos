import { afterEach } from "node:test";
import { JSDOM } from "jsdom";

const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost" });

Object.defineProperties(globalThis, {
  window: { configurable: true, value: dom.window },
  document: { configurable: true, value: dom.window.document },
  navigator: { configurable: true, value: dom.window.navigator },
  HTMLElement: { configurable: true, value: dom.window.HTMLElement },
  Node: { configurable: true, value: dom.window.Node },
  Event: { configurable: true, value: dom.window.Event },
  MouseEvent: { configurable: true, value: dom.window.MouseEvent },
  KeyboardEvent: { configurable: true, value: dom.window.KeyboardEvent },
  getComputedStyle: { configurable: true, value: dom.window.getComputedStyle.bind(dom.window) },
  IS_REACT_ACT_ENVIRONMENT: { configurable: true, value: true, writable: true },
});

const [{ clearMocks }, { cleanup }] = await Promise.all([
  import("@tauri-apps/api/mocks"),
  import("@testing-library/react"),
]);

afterEach(() => {
  cleanup();
  clearMocks();
});
