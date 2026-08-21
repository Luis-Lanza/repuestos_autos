import { createElement } from "react";
import { createRoot } from "react-dom/client";

import { SaleScreen } from "./ui/sales/sale-screen.ts";

const root = document.querySelector<HTMLDivElement>("#root");
if (root) createRoot(root).render(createElement(SaleScreen));
