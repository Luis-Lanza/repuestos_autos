import { createElement } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./ui/app.ts";

const root = document.querySelector<HTMLDivElement>("#root");
if (root) createRoot(root).render(createElement(App));
